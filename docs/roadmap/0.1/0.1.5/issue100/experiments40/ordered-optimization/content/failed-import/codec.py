"""Pinned bounded-frame diagnostic codec and canonical content envelopes."""
import ctypes as C,struct,sys
sys.path.insert(0,'/Users/yifanxu/Ephemeral-AI-Lab/layerfs-issue100-40mb-experiments/tools')
from hashing import blake3
z=C.CDLL('/opt/homebrew/lib/libzstd.dylib');P=C.c_void_p;S=C.c_size_t;I=C.c_int
class Header(C.Structure):
 _fields_=[('content',C.c_ulonglong),('window',C.c_ulonglong),('block',C.c_uint),('type',C.c_uint),('size',C.c_uint),('dictid',C.c_uint),('checksum',C.c_uint),('reserved1',C.c_uint),('reserved2',C.c_uint)]
for name,args,res in [('ZSTD_versionNumber',[],C.c_uint),('ZSTD_isError',[S],C.c_uint),('ZSTD_createCCtx',[],P),('ZSTD_CCtx_reset',[P,I],S),('ZSTD_CCtx_setParameter',[P,I,I],S),('ZSTD_CCtx_refPrefix',[P,P,S],S),('ZSTD_compressBound',[S],S),('ZSTD_compress2',[P,P,S,P,S],S),('ZSTD_createDCtx',[],P),('ZSTD_DCtx_reset',[P,I],S),('ZSTD_DCtx_setParameter',[P,I,I],S),('ZSTD_DCtx_refPrefix',[P,P,S],S),('ZSTD_decompressDCtx',[P,P,S,P,S],S),('ZSTD_getFrameHeader',[P,P,S],S),('ZSTD_findFrameCompressedSize',[P,S],S)]:
 f=getattr(z,name);f.argtypes=args;f.restype=res
assert z.ZSTD_versionNumber()==10507
cc=z.ZSTD_createCCtx();dc=z.ZSTD_createDCtx();assert cc and dc
MAX_RAW=2*1024*1024;MAX_FRAME=MAX_RAW+1024

def check(ok,message='content format'):
 if not ok:raise ValueError(message)
def ck(n):check(not z.ZSTD_isError(n),'zstd error');return n

def encode(raw,base=b'',window=None):
 check(0<len(raw)<=MAX_RAW and len(base)<=MAX_RAW,'encode raw bound');ck(z.ZSTD_CCtx_reset(cc,3))
 for k,v in [(100,3),(101,window or (18 if len(raw)<131072 else 20)),(200,1),(201,1),(202,0),(400,0)]:ck(z.ZSTD_CCtx_setParameter(cc,k,v))
 ck(z.ZSTD_CCtx_refPrefix(cc,base,len(base)));size=z.ZSTD_compressBound(len(raw));buf=C.create_string_buffer(size);n=ck(z.ZSTD_compress2(cc,buf,size,raw,len(raw)));check(n<=MAX_FRAME,'encoded frame bound');return buf.raw[:n]

def decode(frame,size,base=b'',window=20):
 check(0<size<=MAX_RAW and len(base)<=MAX_RAW and 1<=len(frame)<=MAX_FRAME,'decode bounds')
 check(frame[:4]==b'\x28\xb5\x2f\xfd' and len(frame)>=5 and frame[4]&0x1b==0,'frame grammar')
 h=Header();check(z.ZSTD_getFrameHeader(C.byref(h),frame,len(frame))==0,'frame header');check(h.content==size and h.window<=1<<window and h.type==0 and h.dictid==0 and h.checksum==1,'frame contract');check(z.ZSTD_findFrameCompressedSize(frame,len(frame))==len(frame),'trailing frame')
 ck(z.ZSTD_DCtx_reset(dc,3));ck(z.ZSTD_DCtx_setParameter(dc,100,window));ck(z.ZSTD_DCtx_refPrefix(dc,base,len(base)));buf=C.create_string_buffer(size);check(ck(z.ZSTD_decompressDCtx(dc,buf,size,frame,len(frame)))==size,'decoded length');return buf.raw

def canonical(role,raw):
 tag={'small':b'LFS5SML\0\0\1','large':b'LFSWFL1\0','native':b'LFS4CHK\0'}[role]
 body=tag+raw;return b'LFSO\1'+struct.pack('>II',len(body)+4,len(body))+body

def object_id(role,raw):return blake3(b'layerfs/object/v2\0'+canonical(role,raw))
