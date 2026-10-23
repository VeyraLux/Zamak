# linkcheck

`linkcheck` opens a web page, finds its links, and checks them with several threads at once. It exits with code 1 when a link is broken, so you can run it in CI to catch dead links on a documentation site.

```bash
zam run examples/apps/linkcheck -- https://www.python.org/ --threads 8
```

```text
144 links on https://www.python.org/
ok   200 https://docs.python.org/
...
FAIL status 404 https://www.python.org/psf/community-infrastructure
143 ok, 1 broken
```

Options:

- `--threads N`: how many links to check at the same time. The default is 4.
- `--timeout SECONDS`: how long to wait for each request. The default is 10.

The program shows these parts of Zamak working together: `std/http` for HTTPS requests that follow redirects, `http.urljoin` for relative links, `std/re` to find links, and `std/thread` workers that take jobs from one queue and put results on another.
