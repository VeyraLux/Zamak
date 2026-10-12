"""Dependency-free cross-language benchmark driver; invoked by bench-common.ps1."""
import argparse
import datetime
import json
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import sys
import time

N = 64

def gcd_batch(seed):
    total = 0
    for i in range(1, 20001):
        x, y = i * 17 + seed + 1, i * 13 + 7
        while y:
            x, y = y, x % y
        total += x
    return total

def prime_batch(seed):
    count = 0
    for n in range(2, 10001 + seed):
        d, prime = 2, True
        while d <= n // d:
            if n % d == 0:
                prime = False
                break
            d += 1
        if prime:
            count += 1
    return count

def fibonacci_batch(seed):
    total = 0
    for i in range(20000 + seed):
        a, b = 0, 1
        for _ in range((i + seed) % 40):
            a, b = b, a + b
        total += a
    return total

def insertion_sort_batch(seed):
    values = [(seed * 29 + i * 37 + 11) % 1009 for i in range(N)]
    for i in range(1, N):
        value, j = values[i], i
        while j > 0 and values[j - 1] > value:
            values[j] = values[j - 1]
            j -= 1
        values[j] = value
    return sum((i + 1) * values[i] for i in range(N))

def binary_search_batch(seed):
    values = [seed * 3 + i * 7 for i in range(N)]
    found = 0
    for i in range(N * 2):
        target = seed * 3 + ((i * 11 + 5) % (N * 2)) * 7
        lo, hi = 0, N - 1
        while lo <= hi:
            mid = (lo + hi) // 2
            if values[mid] == target:
                found += mid + 1
                break
            if values[mid] < target:
                lo = mid + 1
            else:
                hi = mid - 1
    return found

def arithmetic_sum_batch(seed):
    total = 0
    for i in range(N * 8):
        total += (i + seed + 3) * (i * 5 - seed + 7)
    return total

def recurrence_batch(seed):
    a, b, total = seed + 1, seed + 3, 0
    for _ in range(50000):
        a, b = b, (a * 3 + b * 5 + 17) % 1000003
        total += a
    return total

KERNELS = {
    "gcd": gcd_batch, "prime": prime_batch, "fibonacci": fibonacci_batch,
    "insertion_sort": insertion_sort_batch, "binary_search": binary_search_batch,
    "arithmetic_sum": arithmetic_sum_batch, "recurrence": recurrence_batch,
}

C_SOURCE = r'''
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#define N 64
static int64_t gcd_batch(int64_t seed){int64_t total=0;for(int64_t i=1;i<=20000;++i){int64_t x=i*17+seed+1,y=i*13+7;while(y){int64_t r=x%y;x=y;y=r;}total+=x;}return total;}
static int64_t prime_batch(int64_t seed){int64_t count=0;for(int64_t n=2;n<=10000+seed;++n){int64_t d=2;int prime=1;while(d<=n/d){if(n%d==0){prime=0;break;}++d;}if(prime)++count;}return count;}
static int64_t fibonacci_batch(int64_t seed){int64_t total=0;for(int64_t i=0;i<20000+seed;++i){int64_t a=0,b=1;for(int64_t j=0;j<(i+seed)%40;++j){int64_t next=a+b;a=b;b=next;}total+=a;}return total;}
static int64_t insertion_sort_batch(int64_t seed){int64_t v[N];for(int64_t i=0;i<N;++i)v[i]=(seed*29+i*37+11)%1009;for(int64_t i=1;i<N;++i){int64_t value=v[i],j=i;while(j>0&&v[j-1]>value){v[j]=v[j-1];--j;}v[j]=value;}int64_t total=0;for(int64_t i=0;i<N;++i)total+=(i+1)*v[i];return total;}
static int64_t binary_search_batch(int64_t seed){int64_t v[N],found=0;for(int64_t i=0;i<N;++i)v[i]=seed*3+i*7;for(int64_t i=0;i<N*2;++i){int64_t target=seed*3+((i*11+5)%(N*2))*7,lo=0,hi=N-1;while(lo<=hi){int64_t mid=(lo+hi)/2;if(v[mid]==target){found+=mid+1;break;}if(v[mid]<target)lo=mid+1;else hi=mid-1;}}return found;}
static int64_t arithmetic_sum_batch(int64_t seed){int64_t total=0;for(int64_t i=0;i<N*8;++i)total+=(i+seed+3)*(i*5-seed+7);return total;}
static int64_t recurrence_batch(int64_t seed){int64_t a=seed+1,b=seed+3,total=0;for(int64_t i=0;i<50000;++i){int64_t next=(a*3+b*5+17)%1000003;a=b;b=next;total+=a;}return total;}
int main(int argc,char**argv){if(argc!=3)return 2;int k=atoi(argv[1]);int64_t repeats=atoll(argv[2]),total=0;for(int64_t r=0;r<repeats;++r){int64_t s=r%8;switch(k){case 0:total+=gcd_batch(s);break;case 1:total+=prime_batch(s);break;case 2:total+=fibonacci_batch(s);break;case 3:total+=insertion_sort_batch(s);break;case 4:total+=binary_search_batch(s);break;case 5:total+=arithmetic_sum_batch(s);break;default:total+=recurrence_batch(s);}}printf("%lld\n",(long long)total);return 0;}
'''

RUST_SOURCE = r'''
const N: usize = 64;
fn gcd_batch(seed:i64)->i64{let mut total=0;for i in 1..=20000{let(mut x,mut y)=(i*17+seed+1,i*13+7);while y!=0{let r=x%y;x=y;y=r;}total+=x;}total}
fn prime_batch(seed:i64)->i64{let mut count=0;for n in 2..=10000+seed{let(mut d,mut prime)=(2,true);while d<=n/d{if n%d==0{prime=false;break;}d+=1;}if prime{count+=1;}}count}
fn fibonacci_batch(seed:i64)->i64{let mut total=0;for i in 0..20000+seed{let(mut a,mut b)=(0,1);for _ in 0..(i+seed)%40{let next=a+b;a=b;b=next;}total+=a;}total}
fn insertion_sort_batch(seed:i64)->i64{let mut v=[0i64;N];for i in 0..N{v[i]=(seed*29+i as i64*37+11)%1009;}for i in 1..N{let(value,mut j)=(v[i],i);while j>0&&v[j-1]>value{v[j]=v[j-1];j-=1;}v[j]=value;}let mut total=0;for i in 0..N{total+=(i as i64+1)*v[i];}total}
fn binary_search_batch(seed:i64)->i64{let mut v=[0i64;N];for i in 0..N{v[i]=seed*3+i as i64*7;}let mut found=0;for i in 0..N*2{let target=seed*3+((i*11+5)%(N*2)) as i64*7;let(mut lo,mut hi)=(0i64,N as i64-1);while lo<=hi{let mid=(lo+hi)/2;if v[mid as usize]==target{found+=mid+1;break;}if v[mid as usize]<target{lo=mid+1;}else{hi=mid-1;}}}found}
fn arithmetic_sum_batch(seed:i64)->i64{let mut total=0;for i in 0..N*8{let i=i as i64;total+=(i+seed+3)*(i*5-seed+7);}total}
fn recurrence_batch(seed:i64)->i64{let(mut a,mut b,mut total)=(seed+1,seed+3,0);for _ in 0..50000{let next=(a*3+b*5+17)%1000003;a=b;b=next;total+=a;}total}
fn main(){let args:Vec<String>=std::env::args().collect();let k:usize=args[1].parse().unwrap();let repeats:i64=args[2].parse().unwrap();let mut total=0;for r in 0..repeats{let s=r%8;total+=match k{0=>gcd_batch(s),1=>prime_batch(s),2=>fibonacci_batch(s),3=>insertion_sort_batch(s),4=>binary_search_batch(s),5=>arithmetic_sum_batch(s),_=>recurrence_batch(s)};}println!("{}",total);}
'''

GO_SOURCE = r'''
package main
import("fmt";"os";"strconv")
const N int64=64
func gcd(seed int64)int64{total:=int64(0);for i:=int64(1);i<=20000;i++{x,y:=i*17+seed+1,i*13+7;for y!=0{x,y=y,x%y};total+=x};return total}
func prime(seed int64)int64{count:=int64(0);for n:=int64(2);n<=10000+seed;n++{d:=int64(2);ok:=true;for d<=n/d{if n%d==0{ok=false;break};d++};if ok{count++}};return count}
func fibonacci(seed int64)int64{total:=int64(0);for i:=int64(0);i<20000+seed;i++{a,b:=int64(0),int64(1);for j:=int64(0);j<(i+seed)%40;j++{a,b=b,a+b};total+=a};return total}
func insertion(seed int64)int64{v:=[64]int64{};for i:=int64(0);i<N;i++{v[i]=(seed*29+i*37+11)%1009};for i:=int64(1);i<N;i++{value,j:=v[i],i;for j>0&&v[j-1]>value{v[j]=v[j-1];j--};v[j]=value};total:=int64(0);for i:=int64(0);i<N;i++{total+=(i+1)*v[i]};return total}
func binary(seed int64)int64{v:=[64]int64{};for i:=int64(0);i<N;i++{v[i]=seed*3+i*7};found:=int64(0);for i:=int64(0);i<N*2;i++{target:=seed*3+((i*11+5)%(N*2))*7;lo,hi:=int64(0),N-1;for lo<=hi{mid:=(lo+hi)/2;if v[mid]==target{found+=mid+1;break};if v[mid]<target{lo=mid+1}else{hi=mid-1}}};return found}
func dot(seed int64)int64{total:=int64(0);for i:=int64(0);i<N*8;i++{total+=(i+seed+3)*(i*5-seed+7)};return total}
func recurrence(seed int64)int64{a,b,total:=seed+1,seed+3,int64(0);for i:=0;i<50000;i++{a,b=b,(a*3+b*5+17)%1000003;total+=a};return total}
func main(){k,_:=strconv.Atoi(os.Args[1]);repeats,_:=strconv.ParseInt(os.Args[2],10,64);total:=int64(0);for r:=int64(0);r<repeats;r++{s:=r%8;switch k{case 0:total+=gcd(s);case 1:total+=prime(s);case 2:total+=fibonacci(s);case 3:total+=insertion(s);case 4:total+=binary(s);case 5:total+=dot(s);default:total+=recurrence(s)}};fmt.Println(total)}
'''

def command(args, cwd=None, env=None):
    started = time.perf_counter()
    result = subprocess.run([str(a) for a in args], cwd=cwd, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
    elapsed = time.perf_counter() - started
    if result.returncode:
        raise RuntimeError(f"Command failed ({result.returncode}): {args}\n{result.stdout}\n{result.stderr}")
    return result.stdout.strip(), elapsed, result.stderr

def benchmark(args):
    root = Path(__file__).resolve().parent.parent
    output = root / "results" / ("tutorial-perf-" + datetime.datetime.now().strftime("%Y%m%d-%H%M%S-%f")); output.mkdir(parents=True)
    env = dict(os.environ, ZAMAK_CACHE_DIR=str(output / "zam-cache"))
    template = (root / "examples/tutorial_perf/main.zm").read_text(encoding="utf-8").split("fn main() {")[0]
    builds, rows, calibrations = [], [], []
    def compile_c(label, source, extra):
        path=output/f"{label}.c"; path.write_text(source,encoding='utf-8'); exe=output/f"{label}.exe"
        obj = output / (label + '.obj')
        cmd=[args.cc,'/nologo','/O2','/MT','/utf-8',path,f'/Fe:{exe}',f'/Fo:{obj}']
        if Path(args.cc).stem.lower()!='cl': cmd=[args.cc,'-O2','-std=c11',path,'-o',exe]
        out,sec,err=command(cmd,cwd=output); builds.append(dict(language=label,seconds=sec,command=[str(x) for x in cmd],stdout=out,stderr=err)); return exe
    c_exe=compile_c('common-c',C_SOURCE,'')
    runners={'Zamak':None,'C':c_exe,'Python':Path(__file__).resolve()}
    if args.rustc:
        for label, checked in [('Rust',False),('Rust checked',True)]:
            p=output/f"{label.replace(' ','-')}.rs"; p.write_text(RUST_SOURCE,encoding='utf-8'); exe=output/f"{label.replace(' ','-')}.exe"; cmd=[args.rustc,'-C','opt-level=3','-C',f'overflow-checks={str(checked).lower()}',p,'-o',exe]; out,sec,err=command(cmd); builds.append(dict(language=label,seconds=sec,command=[str(x) for x in cmd],stdout=out,stderr=err)); runners[label]=exe
    if args.go:
        p=output/'common.go'; p.write_text(GO_SOURCE,encoding='utf-8'); exe=output/'common-go.exe'; cmd=[args.go,'build','-trimpath','-o',exe,p]; out,sec,err=command(cmd); builds.append(dict(language='Go',seconds=sec,command=[str(x) for x in cmd],stdout=out,stderr=err)); runners['Go']=exe
    def prepare(lang,kernel,repeats):
        if lang=='Zamak':
            p=output/f'{kernel}_{repeats}.zm'; p.write_text(template+f'''fn main() {{\n    let mut r = 0\n    let mut total = 0\n    while r < {repeats} {{\n        total = total + {kernel}_batch(r % 8)\n        r = r + 1\n    }}\n    println(total)\n}}\n''',encoding='utf-8'); out,sec,err=command([args.zam,'build',p],env=env); builds.append(dict(language='Zamak',kernel=kernel,repeats=repeats,seconds=sec,command=[args.zam,'build',p],stdout=out,stderr=err)); paths=[line[7:] for line in err.splitlines() if line.startswith('OUTPUT ')]; return [paths[0]]
        if lang=='Python': return [sys.executable,runners[lang],'--kernel',kernel,'--repeats',repeats]
        return [runners[lang],list(KERNELS).index(kernel),repeats]
    languages=list(runners)
    selected={k:f for k,f in KERNELS.items() if not args.only or k in args.only}
    for kernel,function in selected.items():
        cycle=sum(function(s) for s in range(8))
        pending={}
        for lang in languages:
            if args.smoke:
                run=prepare(lang,kernel,8); out,elapsed,_=command(run)
                if int(out)!=cycle: raise AssertionError(f'{lang}/{kernel}: {out} != {cycle}')
                print(f'Smoke {kernel}/{lang}: {out}',flush=True)
                rows.append(dict(language=lang,kernel=kernel,checksum=cycle))
                continue
            repeats=8
            while True:
                run=prepare(lang,kernel,repeats); out,elapsed,_=command(run); expected=cycle*(repeats//8)
                if int(out)!=expected: raise AssertionError(f'{lang}/{kernel}: {out} != {expected}')
                calibrations.append(dict(language=lang,kernel=kernel,repeats=repeats,seconds=elapsed,checksum=out))
                if elapsed>=args.min_seconds*1.5: break
                repeats=max(repeats+8,int(repeats*args.min_seconds*2/max(elapsed,0.001)/8+1)*8)
            pending[lang]=dict(run=run,repeats=repeats,expected=expected,samples=[],raw=[])
            print(f'Calibrated {kernel}/{lang}: {repeats} batches, {elapsed:.3f}s', flush=True)
        if args.smoke: continue
        rng=random.Random(list(KERNELS).index(kernel))
        for sample in range(args.samples):
            order=list(languages); rng.shuffle(order)
            for lang in order:
                item=pending[lang]
                out,elapsed,stderr=command(item['run'])
                if int(out)!=item['expected'] or elapsed<args.min_seconds: raise AssertionError(f'{lang}/{kernel}: invalid timed sample')
                item['samples'].append(elapsed); item['raw'].append(dict(stdout=out,stderr=stderr,seconds=elapsed,sample=sample))
        for lang,item in pending.items():
            median=statistics.median(item['samples'])
            rows.append(dict(language=lang,kernel=kernel,repeats=item['repeats'],checksum=item['expected'],command=[str(x) for x in item['run']],samples_seconds=item['samples'],median_seconds=median,median_batch_ms=median*1000/item['repeats'],raw=item['raw']))
            print(f'Median {kernel}/{lang}: {median*1000/item["repeats"]:.6f} ms/batch',flush=True)
    versions={'Python':sys.version,'platform':platform.platform(),'cpu':platform.processor(),'Go':command([args.go,'version'])[0] if args.go else None,'C':dict(path=args.cc,version=subprocess.run([args.cc] if Path(args.cc).stem.lower()=='cl' else [args.cc,'--version'],capture_output=True,text=True,encoding='utf-8',errors='replace').stdout)}
    for name,cmd in [('Zamak',[args.zam,'--version']),('Rust',[args.rustc,'--version'] if args.rustc else None)]:
        if cmd: versions[name]=command(cmd)[0]
    if args.smoke:
        path=output/'smoke.json'; path.write_text(json.dumps(dict(builds=builds,results=rows),indent=2,default=str),encoding='utf-8'); print(f'SMOKE: {path}'); return
    old_criteria={k:next(r['median_batch_ms'] for r in rows if r['kernel']==k and r['language']=='Zamak')<=3*next(r['median_batch_ms'] for r in rows if r['kernel']==k and r['language']=='C') for k in selected}
    medians={(r['kernel'],r['language']):r['median_batch_ms'] for r in rows}
    modern_criteria={k:medians[k,'Zamak'] < medians[k,'Rust'] and medians[k,'Zamak'] < medians[k,'Go'] for k in selected if (k,'Rust') in medians and (k,'Go') in medians}
    report=dict(versions=versions,min_seconds=args.min_seconds,samples=args.samples,seed_cycle=list(range(8)),screening_limit_zamak_over_c=3,screening_pass_legacy=old_criteria,screening_pass_modern=modern_criteria,reference_by_seed={n:[f(s) for s in range(8)] for n,f in KERNELS.items()},methodology='Same seven algorithms, fixed i64 arrays (N=64), seeded checksum cycle, fresh-process wall clock, median of five samples per language/kernel, launch and output included. Rust release opt-level=3 overflow-checks=off; Rust checked diagnoses arithmetic-check overhead. Go is built with the default optimized compiler pipeline. Compile time excluded.',benchmark_c=args.cc,benchmark_zamak_backend=os.environ.get('CC','MSVC default on Windows; cc otherwise'))
    report.update(builds=builds,calibrations=calibrations,results=rows); path=output/'results.json'; path.write_text(json.dumps(report,indent=2,default=str),encoding='utf-8'); print(f'JSON: {path}')

if __name__=='__main__':
    parser=argparse.ArgumentParser(); parser.add_argument('--kernel',choices=KERNELS); parser.add_argument('--repeats',type=int,default=8); parser.add_argument('--zam'); parser.add_argument('--cc'); parser.add_argument('--rustc'); parser.add_argument('--go'); parser.add_argument('--min-seconds',type=float,default=0.5); parser.add_argument('--samples',type=int,default=5); parser.add_argument('--smoke',action='store_true'); parser.add_argument('--only',nargs='+',choices=KERNELS); options=parser.parse_args()
    if options.kernel: print(sum(KERNELS[options.kernel](r%8) for r in range(options.repeats)))
    elif options.zam and options.cc: benchmark(options)
    else: parser.error('Benchmark mode requires --zam and --cc')
