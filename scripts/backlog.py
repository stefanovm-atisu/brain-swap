#!/usr/bin/env python3
"""Backlog tooling for the cloud loop. Standard library only.

  backlog.py next [--limit N] [--human]   tasks ready to start (every dependency done), lowest wave first
  backlog.py show <ID>                    the task block with its feature contract and epic notes
  backlog.py spec <DOC> <REF>             a section ("6.3", "4.8") or an ID line ("FR-41", "I4") of PRD or TECHSPEC
  backlog.py set-status <ID> <status>     rewrite the task's Status line in place
  backlog.py verify <ID>                  every test name listed under Tests first exists in src/ or tests/
  backlog.py check                        structural checks: labels, dependencies, cycles, dashes

Exit code 1 on any problem. `next` and `show` print JSON and text respectively.
"""
import glob
import json
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LABELS = ['Status', 'Depends on', 'Covers', 'Size', 'Scope', 'Not in scope', 'Tests first', 'DoD']


def load():
    """Parse every backlog/E*.md into features and tasks."""
    features, tasks, problems = {}, {}, []
    for path in sorted(glob.glob(os.path.join(ROOT, 'backlog', 'E*.md'))):
        name = os.path.basename(path)
        text = open(path, encoding='utf-8').read()
        for i, line in enumerate(text.splitlines(), 1):
            if '—' in line or '–' in line:
                problems.append(f'{name}:{i}: em or en dash')
        epic_intro = text.split('\n## ', 1)[0]
        parts = re.split(r'^(#{2,3} .*)$', text, flags=re.M)
        feature = None
        for i in range(1, len(parts), 2):
            heading, body = parts[i], parts[i + 1]
            fm = re.match(r'^## (E\d-F\d) (.+)$', heading)
            tm = re.match(r'^### (E\d-F\d-T\d+) (.+)$', heading)
            if fm:
                feature = fm.group(1)
                features[feature] = {'title': fm.group(2).strip(), 'file': name, 'header': heading + body.rstrip() + '\n',
                                     'epic_intro': epic_intro.strip(), 'tasks': []}
            elif tm:
                tid = tm.group(1)
                fields = {}
                for lab in LABELS:
                    m = re.search(rf'^- {re.escape(lab)}:\s*(.*)$', body, re.M)
                    if m:
                        fields[lab] = m.group(1).strip()
                    elif lab != 'Tests first':
                        problems.append(f'{name}: {tid} missing "- {lab}:"')
                human = bool(re.search(r'^- Procedure:', body, re.M))
                if 'Tests first' not in fields and not human:
                    problems.append(f'{name}: {tid} has neither Tests first nor Procedure')
                tests = re.findall(r'^\s+\d+\.\s+`([A-Za-z0-9_]+)`', body, re.M) if not human else []
                tasks[tid] = {'title': tm.group(2).strip(), 'file': name, 'feature': feature, 'fields': fields,
                              'human': human, 'tests': tests, 'block': heading + body.rstrip() + '\n',
                              'deps': re.findall(r'E\d-F\d(?:-T\d+)?', fields.get('Depends on', ''))}
                if feature in features:
                    features[feature]['tasks'].append(tid)
                else:
                    problems.append(f'{name}: {tid} outside any feature')
    return features, tasks, problems


def dep_tasks(tid, tasks, features):
    out = set()
    for d in tasks[tid]['deps']:
        if d in tasks:
            out.add(d)
        elif d in features:
            out.update(features[d]['tasks'])
    return out


def waves(tasks, features, problems):
    wave = {}

    def visit(tid, stack):
        if tid in wave:
            return wave[tid]
        if tid in stack:
            problems.append('dependency cycle: ' + ' -> '.join(stack + [tid]))
            return 0
        stack.append(tid)
        w = 0
        for d in dep_tasks(tid, tasks, features):
            w = max(w, visit(d, stack) + 1)
        stack.pop()
        wave[tid] = w
        return w

    for tid in tasks:
        visit(tid, [])
    return wave


def status(t):
    return t['fields'].get('Status', '').split()[0] if t['fields'].get('Status') else ''


def cmd_next(args):
    limit = int(args[args.index('--limit') + 1]) if '--limit' in args else 3
    human = '--human' in args
    features, tasks, problems = load()
    wave = waves(tasks, features, problems)
    ready = []
    for tid, t in tasks.items():
        if status(t) != 'todo' or t['human'] != human:
            continue
        deps = dep_tasks(tid, tasks, features)
        if any(d not in tasks for d in t['deps'] if d not in features):
            continue
        if all(status(tasks[d]) == 'done' for d in deps):
            ready.append({'id': tid, 'title': t['title'], 'file': t['file'], 'wave': wave[tid],
                          'size': t['fields'].get('Size'), 'depends_on': t['fields'].get('Depends on'),
                          'covers': t['fields'].get('Covers'), 'human': t['human']})
    ready.sort(key=lambda r: (r['wave'], r['id']))
    counts = {s: sum(1 for t in tasks.values() if status(t) == s) for s in ('todo', 'doing', 'done', 'blocked')}
    print(json.dumps({'ready': ready[:limit], 'ready_total': len(ready), 'counts': counts}, indent=1))


def cmd_show(args):
    features, tasks, _ = load()
    t = tasks.get(args[0]) if args else None
    if not t:
        sys.exit(f'unknown task {args[:1]}')
    f = features[t['feature']]
    print(f"# Epic notes ({t['file']})\n\n{f['epic_intro']}\n\n# Feature\n\n{f['header']}\n# Task\n\n{t['block']}")


def cmd_spec(args):
    doc, ref = args[0].upper(), args[1]
    text = open(os.path.join(ROOT, f'{doc}.md'), encoding='utf-8').read()
    m = re.search(rf'^- \*\*{re.escape(ref)}\*\*.*$', text, re.M) or re.search(rf'^- {re.escape(ref)}\b.*$', text, re.M)
    if m:
        print(m.group(0))
        return
    num = re.escape(ref.rstrip('.'))
    m = re.search(rf'^(#+) {num}\.? .*$', text, re.M) or re.search(rf'^(#+) {num}\b.*$', text, re.M)
    if not m:
        sys.exit(f'{doc}: no section or ID {ref}')
    level = len(m.group(1))
    rest = text[m.end():]
    n = re.search(rf'^#{{1,{level}}} ', rest, re.M)
    print(m.group(0) + (rest[:n.start()] if n else rest))


def cmd_set_status(args):
    tid, new = args[0], ' '.join(args[1:])
    features, tasks, _ = load()
    t = tasks.get(tid)
    if not t:
        sys.exit(f'unknown task {tid}')
    path = os.path.join(ROOT, 'backlog', t['file'])
    text = open(path, encoding='utf-8').read()
    old = t['block']
    newblock = re.sub(r'^- Status: .*$', f'- Status: {new}', old, count=1, flags=re.M)
    if old not in text:
        sys.exit(f'{tid}: block not found verbatim')
    open(path, 'w', encoding='utf-8').write(text.replace(old, newblock, 1))
    print(f'{tid}: Status: {new}')


def cmd_verify(args):
    features, tasks, _ = load()
    t = tasks.get(args[0]) if args else None
    if not t:
        sys.exit(f'unknown task {args[:1]}')
    if t['human']:
        print(f'{args[0]}: human task, nothing to verify')
        return
    missing = []
    for name in t['tests']:
        r = subprocess.run(['grep', '-rlE', rf'fn\s+{name}\s*\(', 'src', 'tests'], cwd=ROOT, capture_output=True)
        if r.returncode != 0:
            missing.append(name)
    if missing:
        sys.exit(f"{args[0]}: tests missing in src/ or tests/: {', '.join(missing)}")
    print(f"{args[0]}: all {len(t['tests'])} listed tests exist")


def cmd_check(_args):
    features, tasks, problems = load()
    waves(tasks, features, problems)
    ids = set(tasks) | set(features)
    for tid, t in tasks.items():
        for d in t['deps']:
            if d not in ids:
                problems.append(f'{t["file"]}: {tid} depends on unknown {d}')
        if status(t) not in ('todo', 'doing', 'done', 'blocked'):
            problems.append(f'{t["file"]}: {tid} bad status {t["fields"].get("Status")!r}')
    print(f'features {len(features)}, tasks {len(tasks)}, problems {len(problems)}')
    for p in problems:
        print('  ' + p)
    if problems:
        sys.exit(1)


if __name__ == '__main__':
    cmds = {'next': cmd_next, 'show': cmd_show, 'spec': cmd_spec, 'set-status': cmd_set_status,
            'verify': cmd_verify, 'check': cmd_check}
    if len(sys.argv) < 2 or sys.argv[1] not in cmds:
        sys.exit(__doc__)
    cmds[sys.argv[1]](sys.argv[2:])
