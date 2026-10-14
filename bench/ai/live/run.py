"""Live AI benchmark: the same model writes the same small CLI programs in Zamak, Rust and Go.

Each trial loops: ask the model for a complete program -> build -> run the tests -> feed the first
compiler error or failing test back, for at most --rounds rounds. Builds and test runs are
serialized behind a lock so concurrent model calls cannot distort the compile timings.

Credentials: ZAM_LLM_BASE_URL / ZAM_LLM_API_KEY, or a cc-switch provider via --provider (read at
runtime from ~/.cc-switch/cc-switch.db; the key is never printed or written to the results).

    python bench/ai/live/run.py --verify-solutions
    python bench/ai/live/run.py --provider kouri --models deepseek-v4.1-flash claude-opus-5-5-code
"""
import argparse
import concurrent.futures
import datetime
import json
import os
import re
import shutil
import sqlite3
import statistics
import subprocess
import threading
import time
import urllib.error
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
LANGUAGES = ['zamak', 'rust', 'go']
EXTENSION = {'zamak': 'zm', 'rust': 'rs', 'go': 'go'}
PROGRESS = re.compile(r'^(CACHE|OBJECT|LINK|TIMING|OUTPUT) ')
BUILD_LOCK = threading.Lock()


def credentials(provider):
    base = os.environ.get('ZAM_LLM_BASE_URL')
    key = os.environ.get('ZAM_LLM_API_KEY')
    if base and key:
        return base.rstrip('/'), key
    if not provider:
        raise SystemExit('set ZAM_LLM_BASE_URL and ZAM_LLM_API_KEY, or pass --provider')
    db = Path.home() / '.cc-switch' / 'cc-switch.db'
    con = sqlite3.connect(f'file:{db}?mode=ro', uri=True)
    row = con.execute(
        "select settings_config from providers where lower(name)=? and app_type='pi'",
        (provider.lower(),)).fetchone()
    if not row:
        raise SystemExit(f'no OpenAI-compatible cc-switch provider named {provider}')
    config = json.loads(row[0])
    return config['baseUrl'].rstrip('/'), config['apiKey']


def system_prompt(language):
    common = ('You write complete single-file command-line programs. Reply with exactly one fenced '
              'code block containing the whole program and nothing else.')
    if language == 'rust':
        return common + ' Language: Rust, edition 2021, standard library only.'
    if language == 'go':
        return common + ' Language: Go, package main, standard library only.'
    reference = (HERE / 'zamak-reference.md').read_text(encoding='utf-8')
    return (common + ' Language: Zamak, a new language you have not seen before. '
            'Its complete reference follows.\n\n' + reference)


def chat(base, key, model, messages, max_tokens):
    body = json.dumps({'model': model, 'messages': messages, 'temperature': 0,
                       'max_tokens': max_tokens}).encode()
    for attempt in range(4):
        request = urllib.request.Request(f'{base}/chat/completions', data=body, headers={
            'Authorization': f'Bearer {key}', 'Content-Type': 'application/json'})
        started = time.perf_counter()
        try:
            with urllib.request.urlopen(request, timeout=300) as response:
                data = json.load(response)
            return (data['choices'][0]['message'].get('content') or '',
                    time.perf_counter() - started, data.get('usage') or {})
        except (urllib.error.URLError, TimeoutError, KeyError, json.JSONDecodeError) as error:
            if attempt == 3:
                raise RuntimeError(f'model call failed: {error}') from None
            time.sleep(5 * (attempt + 1))


def extract(reply):
    blocks = re.findall(r'```[A-Za-z0-9_+-]*\r?\n(.*?)```', reply, re.S)
    return max(blocks, key=len) if blocks else reply.strip() + '\n'


def build(language, directory, zam, cache):
    source = directory / f'main.{EXTENSION[language]}'
    exe = directory / ('main.exe' if os.name == 'nt' else 'main')
    if language == 'zamak':
        command = [zam, 'build', source.name]
    elif language == 'rust':
        command = ['rustc', '--edition', '2021', '-o', exe.name, source.name]
    else:
        command = ['go', 'build', '-o', exe.name, source.name]
    env = dict(os.environ, ZAMAK_CACHE_DIR=str(cache))
    started = time.perf_counter()
    result = subprocess.run(command, cwd=directory, capture_output=True, env=env)
    seconds = time.perf_counter() - started
    stderr = result.stderr.decode('utf-8', 'replace')
    stdout = result.stdout.decode('utf-8', 'replace')
    if language == 'zamak':
        outputs = [line[7:].strip() for line in stderr.splitlines() if line.startswith('OUTPUT ')]
        exe = (directory / outputs[-1]) if outputs else exe
        stderr = '\n'.join(line for line in stderr.splitlines() if not PROGRESS.match(line))
    diagnostics = (stdout + stderr).strip()
    return result.returncode == 0, seconds, diagnostics, exe


def run_tests(exe, tests):
    for index, test in enumerate(tests):
        try:
            result = subprocess.run([str(exe)] + test.get('args', []), capture_output=True,
                                    input=test.get('stdin', '').encode(), timeout=20)
        except subprocess.TimeoutExpired:
            return index, 'timed out after 20 seconds', None
        stdout = result.stdout.decode('utf-8', 'replace').replace('\r\n', '\n')
        expected_exit = test.get('exit', 0)
        if stdout != test['stdout'] or result.returncode != expected_exit:
            return index, stdout, result
    return None, None, None


def describe_failure(task, index, stdout, result):
    test = task['tests'][index]
    lines = [f'Test {index + 1} failed.']
    if test.get('args'):
        lines.append(f'Arguments: {json.dumps(test["args"])}')
    lines.append(f'Standard input:\n```\n{test.get("stdin", "")}```')
    lines.append(f'Expected standard output:\n```\n{test["stdout"]}```')
    lines.append(f'Expected exit status: {test.get("exit", 0)}')
    if result is None:
        lines.append(stdout)
    else:
        lines.append(f'Actual standard output:\n```\n{stdout[:2000]}```')
        lines.append(f'Actual exit status: {result.returncode}')
        stderr = result.stderr.decode('utf-8', 'replace')[:2000]
        if stderr:
            lines.append(f'Actual standard error:\n```\n{stderr}```')
    lines.append('Reply with the corrected full program.')
    return '\n'.join(lines)


def trial(args, base, key, model, language, task, out, zam, cache):
    messages = [{'role': 'system', 'content': system_prompt(language)},
                {'role': 'user', 'content': f'Task: {task["spec"]}'}]
    record = {'model': model, 'language': language, 'task': task['id'], 'rounds': [],
              'solved': False}
    for number in range(1, args.rounds + 1):
        directory = out / model / language / task['id'] / f'round-{number}'
        directory.mkdir(parents=True, exist_ok=True)
        reply, model_seconds, usage = chat(base, key, model, messages, args.max_tokens)
        code = extract(reply)
        (directory / f'main.{EXTENSION[language]}').write_text(code, encoding='utf-8')
        with BUILD_LOCK:
            ok, build_seconds, diagnostics, exe = build(language, directory, zam, cache)
            failing = run_tests(exe, task['tests']) if ok else (None, None, None)
        entry = {'round': number, 'model_seconds': round(model_seconds, 3),
                 'build_seconds': round(build_seconds, 3), 'built': ok,
                 'prompt_tokens': usage.get('prompt_tokens', 0),
                 'completion_tokens': usage.get('completion_tokens', 0)}
        if not ok:
            entry['outcome'] = 'compile_error'
            feedback = (f'The program failed to compile:\n```\n{diagnostics[:4000]}\n```\n'
                        'Reply with the corrected full program.')
        elif failing[0] is not None:
            entry['outcome'] = 'test_failed'
            feedback = describe_failure(task, *failing)
        else:
            entry['outcome'] = 'passed'
        record['rounds'].append(entry)
        if entry['outcome'] == 'passed':
            record['solved'] = True
            break
        messages += [{'role': 'assistant', 'content': reply}, {'role': 'user', 'content': feedback}]
    return record


def summarize(records, out):
    lines = ['| model | language | solved | first try | rounds (solved) | compile errors | '
             'build s / round (median) | build s total | model s total | tokens |',
             '|---|---|---|---|---|---|---|---|---|---|']
    for model in sorted({r['model'] for r in records}):
        for language in LANGUAGES:
            group = [r for r in records if r['model'] == model and r['language'] == language]
            if not group:
                continue
            rounds = [e for r in group for e in r['rounds']]
            solved = [r for r in group if r['solved']]
            first = sum(1 for r in group if r['solved'] and len(r['rounds']) == 1)
            mean_rounds = statistics.mean(len(r['rounds']) for r in solved) if solved else 0
            errors = sum(1 for e in rounds if e['outcome'] == 'compile_error')
            builds = [e['build_seconds'] for e in rounds]
            tokens = sum(e['prompt_tokens'] + e['completion_tokens'] for e in rounds)
            lines.append(
                f'| {model} | {language} | {len(solved)}/{len(group)} | {first}/{len(group)} | '
                f'{mean_rounds:.2f} | {errors} | {statistics.median(builds):.2f} | '
                f'{sum(builds):.1f} | {sum(e["model_seconds"] for e in rounds):.0f} | {tokens} |')
    table = '\n'.join(lines) + '\n'
    (out / 'summary.md').write_text(table, encoding='utf-8')
    return table


def verify_solutions(zam):
    tasks = json.loads((HERE / 'tasks.json').read_text(encoding='utf-8'))
    work = ROOT / 'results' / 'ai-live-verify'
    shutil.rmtree(work, ignore_errors=True)
    failed = 0
    for task in tasks:
        directory = work / task['id']
        directory.mkdir(parents=True)
        shutil.copy(HERE / 'solutions' / f'{task["id"]}.zm', directory / 'main.zm')
        ok, seconds, diagnostics, exe = build('zamak', directory, zam, work / 'cache')
        status = 'build failed: ' + diagnostics if not ok else None
        if ok:
            index, stdout, result = run_tests(exe, task['tests'])
            if index is not None:
                status = describe_failure(task, index, stdout, result)
        print(f'{task["id"]}: {"ok" if status is None else "FAIL"} ({seconds:.2f}s build)')
        if status:
            print(status)
            failed += 1
    raise SystemExit(1 if failed else 0)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--provider')
    parser.add_argument('--models', nargs='+', default=['deepseek-v4.1-flash'])
    parser.add_argument('--languages', nargs='+', default=LANGUAGES, choices=LANGUAGES)
    parser.add_argument('--tasks', nargs='+')
    parser.add_argument('--rounds', type=int, default=5)
    parser.add_argument('--max-tokens', type=int, default=8192)
    parser.add_argument('--jobs', type=int, default=6)
    parser.add_argument('--zam', default=shutil.which('zam') or 'zam')
    parser.add_argument('--verify-solutions', action='store_true')
    args = parser.parse_args()
    # Builds run inside per-trial directories, so a relative compiler path must be made absolute.
    if os.sep in args.zam or '/' in args.zam:
        args.zam = str(Path(args.zam).resolve())
    if args.verify_solutions:
        verify_solutions(args.zam)
    base, key = credentials(args.provider)
    tasks = json.loads((HERE / 'tasks.json').read_text(encoding='utf-8'))
    if args.tasks:
        tasks = [t for t in tasks if t['id'] in args.tasks]
    stamp = datetime.datetime.now().strftime('%Y%m%d-%H%M%S')
    out = ROOT / 'results' / f'ai-live-{stamp}'
    out.mkdir(parents=True)
    cache = out / 'zam-cache'
    versions = {
        'zam': subprocess.run([args.zam, '--version'], capture_output=True, text=True).stdout.strip(),
        'rustc': subprocess.run(['rustc', '--version'], capture_output=True, text=True).stdout.strip(),
        'go': subprocess.run(['go', 'version'], capture_output=True, text=True).stdout.strip(),
    }
    (out / 'meta.json').write_text(json.dumps({'base_url': base, 'models': args.models,
                                               'rounds': args.rounds, 'versions': versions},
                                              indent=2), encoding='utf-8')
    jobs = [(m, l, t) for m in args.models for l in args.languages for t in tasks]
    records = []
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool, \
            open(out / 'trials.jsonl', 'w', encoding='utf-8') as log:
        futures = {pool.submit(trial, args, base, key, m, l, t, out, args.zam, cache): (m, l, t['id'])
                   for m, l, t in jobs}
        for future in concurrent.futures.as_completed(futures):
            model, language, task = futures[future]
            try:
                record = future.result()
            except Exception as error:
                record = {'model': model, 'language': language, 'task': task, 'rounds': [],
                          'solved': False, 'error': str(error)}
            records.append(record)
            log.write(json.dumps(record) + '\n')
            log.flush()
            print(f'{model} {language} {task}: {"solved" if record["solved"] else "unsolved"} '
                  f'in {len(record["rounds"])} round(s)', flush=True)
    print(summarize([r for r in records if 'error' not in r], out))
    print(f'results: {out}')


if __name__ == '__main__':
    main()
