"""Offline MP2K rendering of two local-ROM themes; never opens an audio device."""
import argparse, array, hashlib, json, os, pathlib, struct, subprocess, sys, wave
from host_tools import CC, ENV
ROOT=pathlib.Path(__file__).resolve().parents[1]
TOOL=ROOT/'reference/gba-audio-tools'
REV='45e84ba8a5a47b22e56dac7e4e87b1ac605bcb19'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--rom', type=pathlib.Path, default=ROOT.parent /
    'FireRedLeafGreenRecomp/variants/firered/roms/firered_usa.gba')
rom_path = parser.parse_args().rom
if not rom_path.is_file():
    parser.error(f'FireRed USA rev0 ROM not found: {rom_path}')
rom = rom_path.read_bytes()
rom_sha1 = hashlib.sha1(rom).hexdigest()
if rom_sha1 != '41cb23d8dccc8ebd7c649cd8fbb58eeace6e2fdc':
    parser.error('Expected a FireRed USA rev0 ROM.')

assert subprocess.check_output(['git','-C',str(TOOL),'rev-parse','HEAD'],text=True).strip()==REV
library=TOOL/'src/gba_audio/_native-local.pyd'
if os.name == 'nt' and not library.exists():
    core=TOOL/'core'
    sources=['extractor/extractor.c','extractor/extractor_mp2k.c','extractor/extractor_webfoot.c','mp2k/mp2k.c','webfoot/webfoot.c']
    subprocess.run([CC,'-shared','-O2','-DGBA_AUDIO_BUILD_SHARED','-I'+str(core),*[str(core/s) for s in sources],'-o',str(library)],env=ENV,check=True)
sys.path.insert(0,str(TOOL/'src'))
from gba_audio.native import extract_songs, FMT_MP2K
from gba_audio.wav import render_pak_song
out=ROOT/'assets/music';out.mkdir(parents=True,exist_ok=True)
provenance={'renderer':'https://github.com/mudassarzahid/gba-audio-tools','revision':REV,'rom_sha1':rom_sha1,'songs':{}}
for index,name,pointer in [(300,'pallet',0x086dd844),(291,'route1',0x086d1ddc)]:
    assert struct.unpack_from('<I',rom,0x4a32cc+index*8)[0]==pointer
    pak,kind=extract_songs(rom,[index],0x4a32cc);assert kind==FMT_MP2K
    path=out/(name+'.wav');seconds=render_pak_song(pak,0,str(path),loop_count=1,fade_ms=0,max_seconds=120)
    with wave.open(str(path),'rb') as w:
        assert w.getnchannels()==2 and w.getsampwidth()==2
        pcm=array.array('h',w.readframes(w.getnframes()));peak=max(abs(v) for v in pcm)
        assert seconds>10 and peak>100
        provenance['songs'][name]={'song_id':index,'seconds':seconds,'sample_rate':w.getframerate(),'peak':peak,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
    print(name,round(seconds,2),'seconds; non-silent stereo WAV rendered')
(ROOT/'data/music-provenance.json').write_text(json.dumps(provenance,indent=2))
