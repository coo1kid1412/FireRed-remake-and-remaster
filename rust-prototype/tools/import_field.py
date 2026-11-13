"""Assemble pinned original field-script macros as data for the native Rust VM.

The host assembler only emits data, not executable guest instructions. Text
payloads use symbolic native strings; native C pointers remain explicit imports.
"""
import hashlib
import json
import re
import struct
import subprocess
import sys
import tempfile
from pathlib import Path
from host_tools import CC, ENV, MINGW

ROOT = Path(__file__).resolve().parents[1]
REF = ROOT/'reference/pokefirered'
REV = '037335f4c725d7c9aecdac87066f2002b4bd7e14'
assert subprocess.check_output(['git','-C',str(REF),'rev-parse','HEAD'],text=True).strip() == REV
sources, texts, fragments = {}, {}, []
macos = sys.platform == 'darwin'

def read(path):
    raw = (REF/path).read_bytes()
    sources[path] = hashlib.sha256(raw).hexdigest()
    return raw.decode('utf-8')

def flatten(path):
    result, label = [], None
    for raw in read(path).splitlines():
        inc = re.fullmatch(r'\s*\.include "([^"]+)"\s*', raw)
        if inc:
            result.append(flatten(inc[1])); continue
        # ARM GAS's @ comment marker is not recognised by the host GAS.
        line = re.split(r'@(?=(?:[^"]*"[^"]*")*[^"]*$)',raw,maxsplit=1)[0]
        match = re.match(r'\s*(\w+)::?\s*$',line)
        if match: label = match[1]
        string = re.fullmatch(r'\s*\.(?:string|braille) "(.*)"\s*',line)
        if string:
            if label is None: raise ValueError(f'Unlabelled text in {path}')
            marker = f'FIELD_NATIVE_TEXT_{len(fragments)}'
            fragments.append((marker,label,string[1]))
            line = f'{marker}:\n.byte 0'  # Conditional assembly selects native text fragments too.
        line = line.replace('::',':')
        if line.strip().startswith('.section'): line = '.data'
        result.append(line)
    return '\n'.join(result)+'\n'

groups = json.loads(read('data/maps/map_groups.json'))
generated, local_ids = [], []
for group_index, group in enumerate(groups['group_order']):
    for map_index, name in enumerate(groups[group]):
        map_data = json.loads(read(f'data/maps/{name}/map.json'))
        generated.append(f'#define {map_data["id"]} {group_index*256+map_index}')
        for index, obj in enumerate(map_data.get('object_events',[]),1):
            if obj.get('local_id'): local_ids.append(f'#define {obj["local_id"]} {index}')

flat = '#define FIRERED 1\n#define REVISION 0\n' + flatten('data/event_scripts.s')
flat = re.sub(r'\.ifdef\s+FIRERED', '.if 1', flat)
flat = re.sub(r'\.ifdef\s+LEAFGREEN', '.if 0', flat)
# Track transitive C header dependencies, including the generated header inputs.
with tempfile.TemporaryDirectory(dir=ROOT/'tools',prefix='field-import-') as temp:
    temp = Path(temp)
    (temp/'constants').mkdir()
    (temp/'constants/map_groups.h').write_text('\n'.join(generated)+'\n')
    (temp/'constants/map_event_ids.h').write_text('\n'.join(local_ids)+'\n')
    layouts = json.loads(read('data/layouts/layouts.json'))['layouts']
    (temp/'constants/layouts.h').write_text('\n'.join(
        f'#define {layout["id"]} {i}' for i,layout in enumerate(layouts,1) if 'id' in layout)+'\n')
    heals = json.loads(read('src/data/heal_locations.json'))['heal_locations']
    (temp/'constants/heal_locations.h').write_text('\n'.join(
        f'#define {location["id"]} {i}' for i,location in enumerate(heals,1))+'\n')
    processed = subprocess.run([CC,'-E','-P','-H','-x','assembler-with-cpp',
        '-I',str(temp),'-I',str(REF/'include'),'-'],input=flat,text=True,env=ENV,capture_output=True,check=True)
    pre = processed.stdout
    for header in re.findall(r'^\.+ (.*)$',processed.stderr,re.M):
        path = Path(header)
        if path.is_relative_to(REF): read(path.relative_to(REF).as_posix())
    pre = re.sub(r'\benum\s*\{.*?\};', '', pre, flags=re.S)
    pre = re.sub(r'[ \t]*([+*/|&<>-])[ \t]*',r'\1',pre)
    if macos:
        # GAS accepts the original symbolic comparisons, but LLVM requires
        # string comparisons for trainer script labels and literal STR_VAR IDs.
        pre = pre.replace(r'.if \event_script == FALSE', r'.ifc \event_script,FALSE')
        stringvar = r'(\.macro stringvar id:req\n).*?(\.endm)'
        pre = re.sub(stringvar, lambda m: m[1] + r'''.ifc \id,STR_VAR_1
    .byte 0
.else
    .ifc \id,STR_VAR_2
        .byte 1
    .else
        .ifc \id,STR_VAR_3
            .byte 2
        .else
            .byte \id
        .endif
    .endif
.endif
''' + m[2], pre, count=1, flags=re.S)
    # GAS accepts .2byte/.4byte with identical little-endian data widths.
    asm = temp/'field.s'; obj = temp/'field.o'; binary = temp/'field.bin'
    asm.write_text(pre,encoding='utf-8')
    if macos:
        # Use 32-bit absolute relocations. The object supplies script data
        # only and is never linked into the Mac binary.
        subprocess.run([CC,'-target','i386-unknown-linux-gnu','-c','-x','assembler',
            str(asm),'-o',str(obj)],env=ENV,check=True)
        nm_tool = ['xcrun','llvm-nm']
        objdump = ['xcrun','llvm-objdump']
        section_dump = subprocess.check_output(objdump+['-s','--section=.data',str(obj)],text=True,env=ENV)
        groups = re.findall(r'^\s+[0-9a-f]+ ((?:[0-9a-f]{2,8} ?){1,4})\s{2,}', section_dump, re.M)
        code = bytearray.fromhex(' '.join(groups))
    else:
        subprocess.run([f'{MINGW}/as.exe','--32',str(asm),'-o',str(obj)],env=ENV,check=True)
        subprocess.run([f'{MINGW}/objcopy.exe','-O','binary','-j','.data',str(obj),str(binary)],env=ENV,check=True)
        code = bytearray(binary.read_bytes())
        nm_tool = [f'{MINGW}/nm.exe']
        objdump = [f'{MINGW}/objdump.exe']
    nm = subprocess.check_output(nm_tool+['--defined-only',str(obj)],text=True,env=ENV)
    labels = {name:int(address,16) for address,kind,name in re.findall(r'^(\w+) ([dD]) (\w+)$',nm,re.M)}
    constants = {name:int(address,16) for address,kind,name in re.findall(r'^(\w+) ([aA]) (\w+)$',nm,re.M)}
    relocs = subprocess.check_output(objdump+['-r',str(obj)],text=True,env=ENV)
    rows = re.findall(r'^([0-9a-f]+)\s+(\w+)\s+([^\s]+)\s*$',relocs,re.M)
    external = {}
    for offset, kind, name in rows:
        if kind.lower() not in ('dir32', 'r_386_32'):
            raise ValueError(f'Unexpected relocation {kind}')
        offset = int(offset,16)
        if name == '.data': continue
        if name not in labels:
            if name not in external: external[name] = 0x80000000 + len(external)*4
            value = external[name]
        else: value = labels[name]
        value += struct.unpack_from('<I',code,offset)[0]
        struct.pack_into('<I',code,offset,value)

handlers = []
for marker, name, value in fragments:
    if marker in labels and name in labels: texts[name] = texts.get(name,'') + value
labels = {name:value for name,value in labels.items() if not name.startswith('FIELD_NATIVE_TEXT_')}
start, end = labels['gScriptCmdTable'], labels['gScriptCmdTableEnd']
by_address = {address:name for name,address in external.items()}
for offset in range(start,end,4):
    handlers.append(by_address[struct.unpack_from('<I',code,offset)[0]])
std = [struct.unpack_from('<I',code,n)[0] for n in range(labels['gStdScripts'],labels['gStdScriptsEnd'],4)]
# Hash original constants headers used by preprocessing; no generated source code.
for header in re.findall(r'#include "([^"]+)"',read('data/event_scripts.s')):
    if (REF/'include'/header).exists(): read('include/'+header)
for path in ['src/script.c','src/scrcmd.c','include/script.h','src/script_pokemon_util.c','src/money.c']:
    read(path)
data = dict(revision=REV, bytes=list(code),labels=labels,constants=constants,
    imports={str(address):name for name,address in external.items()},handlers=handlers,std=std,
    texts={str(labels[name]):value for name,value in texts.items()})
(ROOT/'data/field.json').write_text(json.dumps(data,separators=(',',':'))+'\n',encoding='utf-8')
report = 'field-import-report.json' if macos else 'field-provenance.json'
(ROOT/'data'/report).write_text(json.dumps(dict(revision=REV,sources=sources,
    status='Original macro-assembled field data. Native text adapter; new VM behaviour unverified.',
    bytes=len(code),labels=len(labels),texts=len(texts),handlers=len(handlers)),indent=2)+'\n')
print(f'Imported {len(code)} bytes, {len(labels)} labels, {len(texts)} texts, {len(handlers)} command handlers.')
