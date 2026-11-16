"""macOS companion to measure_memory.py; requires an isolated prepared demo binary."""
import ctypes, hashlib, json, os, plistlib, signal, statistics, subprocess, sys, time
from pathlib import Path
import argparse
parser=argparse.ArgumentParser(description='Native macOS UI memory probe; prepare the source with measure_memory.py first.')
parser.add_argument('binary',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--label',required=True)
args=parser.parse_args();label=args.label
out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
app=out/'MemoryProbe.app';contents=app/'Contents';(contents/'MacOS').mkdir(parents=True)
binary=args.binary.resolve();(contents/'MacOS/clickpulse').write_bytes(binary.read_bytes());(contents/'MacOS/clickpulse').chmod(0o755)
plist={'CFBundleExecutable':'clickpulse','CFBundleIdentifier':'app.clickpulse.memory.'+label,'CFBundleName':'ClickPulse Memory Probe','CFBundlePackageType':'APPL','LSUIElement':True,'NSHighResolutionCapable':True,'CFBundleVersion':'1','CFBundleShortVersionString':label}
(contents/'Info.plist').write_bytes(plistlib.dumps(plist))
subprocess.run(['codesign','--force','--sign','-',str(app)],check=True,capture_output=True)
lib=ctypes.CDLL('/usr/lib/libproc.dylib');lib.proc_pidinfo.argtypes=[ctypes.c_int,ctypes.c_int,ctypes.c_uint64,ctypes.c_void_p,ctypes.c_int]
lib.proc_pid_rusage.argtypes=[ctypes.c_int,ctypes.c_int,ctypes.c_void_p]
def coalition(pid):
 b=(ctypes.c_uint64*5)();n=lib.proc_pidinfo(pid,20,0,b,40)
 return list(b)[:2] if n==40 else None
def procs():
 rows=subprocess.check_output(['ps','-axo','pid=,comm='],text=True).splitlines()
 return [(int(row.strip().split(None,1)[0]),row.strip().split(None,1)[1]) for row in rows if len(row.strip().split(None,1))==2]
class Timebase(ctypes.Structure):
 _fields_=[('numer',ctypes.c_uint32),('denom',ctypes.c_uint32)]
timebase=Timebase()
ctypes.CDLL('/usr/lib/libSystem.B.dylib').mach_timebase_info(ctypes.byref(timebase))
def mem(pid):
 b=ctypes.create_string_buffer(4096)
 if lib.proc_pid_rusage(pid,0,b):return None
 v=(ctypes.c_uint64*10).from_buffer(b,16)
 return {'rss_bytes':v[6],'footprint_bytes':v[7],'cpu_ns':(v[0]+v[1])*timebase.numer/timebase.denom}
subprocess.run(['open','-n','-g','--env','CLICKPULSE_MEMORY_PROBE='+str(out),str(app),'--args','--demo','--background'],check=True)
start=time.monotonic()
while not (out/'pid').exists():
 if time.monotonic()-start>60:raise RuntimeError('Probe failed to launch')
 time.sleep(.25)
pid=int((out/'pid').read_text(encoding='utf-8'));group=coalition(pid)
assert group and group!=coalition(os.getpid()),'Launch must isolate the app coalition'
report={'platform':subprocess.check_output(['sw_vers'],text=True),'architecture':'arm64','source_version':label,'pid':pid,'coalition':group,'demo':True,'note':'Controlled UI scenarios, no recording or user preferences; includes every process in the app resource coalition. macOS footprint includes compressed memory. The renderer group includes WebKit and its audio helpers, even when reparented.','binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'states':{}}
try:
 for stage in ['background','full_en','full_zh','mini','hidden_10m','reopened']:
  while not (out/(stage+'.ready')).exists():
   if time.monotonic()-start>1100:raise RuntimeError('Probe scenario timed out: '+stage)
   if mem(pid) is None:raise RuntimeError('Probe exited: '+stage)
   time.sleep(1)
  lifecycle=(out/'lifecycle').exists()
  if not (lifecycle and stage=='background') and label!='0.3.0' and not (out/'ui-ready').exists(): raise RuntimeError('WebView did not acknowledge the loaded interface')
  if lifecycle and stage=='hidden_10m' and not (out/'disposed').exists(): raise RuntimeError('Hidden renderer was not disposed')
  if lifecycle and stage=='reopened' and int((out/'generation').read_text(encoding='utf-8'))<2: raise RuntimeError('Renderer did not recreate')
  if lifecycle and stage in ['full_en','mini','reopened']:
   assets=json.loads((out/'assets.json').read_text(encoding='utf-8'))
   if any(image['width']!=128 for image in assets['images']) or not all(font['loaded'] for font in assets['fonts']): raise RuntimeError('Native resource failure: '+json.dumps(assets))
  samples=[]
  for _ in range(5):
   members=[{'pid':p,'name':n,**m} for p,n in procs() if coalition(p)==group and (m:=mem(p))]
   samples.append({'at_ns':time.monotonic_ns(),'processes':members,'main_bytes':sum(p['footprint_bytes'] for p in members if p['pid']==pid),'webview_bytes':sum(p['footprint_bytes'] for p in members if p['pid']!=pid),'total_bytes':sum(p['footprint_bytes'] for p in members)})
   time.sleep(1)
  ids=[str(p['pid']) for p in samples[-1]['processes']]
  fp=subprocess.run(['footprint','--noCategories','-f','bytes',*ids],text=True,capture_output=True)
  (out/(stage+'-footprint.txt')).write_text(fp.stdout+fp.stderr, encoding='utf-8')
  report['states'][stage]={'samples':samples,'median_main_bytes':statistics.median(s['main_bytes'] for s in samples),'median_webview_bytes':statistics.median(s['webview_bytes'] for s in samples),'median_total_bytes':statistics.median(s['total_bytes'] for s in samples)}
  first,last=samples[0],samples[-1]
  initial={p['pid']:p['cpu_ns'] for p in first['processes']}
  report['states'][stage]['main_cpu_percent']=100*sum(p['cpu_ns']-initial[p['pid']] for p in last['processes'] if p['pid']==pid and p['pid'] in initial)/(last['at_ns']-first['at_ns'])
  if lifecycle and stage=='background' and any(row['webview_bytes'] for row in samples): raise RuntimeError('Background launch created WebKit processes')
  if lifecycle and stage=='reopened': report['reopen_ready_ms']=int((out/'ui-ready').read_text(encoding='utf-8'))-int((out/'reopened.started').read_text(encoding='utf-8'))
  (out/'measurements.json').write_text(json.dumps(report,indent=2)+'\n', encoding='utf-8')
  (out/(stage+'.measured')).write_text('ok', encoding='utf-8')
  print(stage,round(report['states'][stage]['median_total_bytes']/2**20,1),'MiB',flush=True)
 deadline=time.monotonic()+15
 while mem(pid) is not None and time.monotonic()<deadline:time.sleep(.1)
 if mem(pid) is not None:raise RuntimeError('Probe did not exit after Save and quit')
 report['normal_quit_requested']=(out/'quit-requested').exists()
 report['exited_after_quit']=True
 (out/'measurements.json').write_text(json.dumps(report,indent=2)+'\n', encoding='utf-8')
finally:
 if mem(pid) is not None:
  command=subprocess.check_output(['ps','-p',str(pid),'-o','comm='],text=True).strip()
  if command==str(contents/'MacOS/clickpulse'):os.kill(pid,signal.SIGTERM)
print('Measurement complete',out,flush=True)
