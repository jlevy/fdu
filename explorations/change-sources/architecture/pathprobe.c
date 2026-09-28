// Firmlink/path-filter probe: create a device-relative FSEvents stream for ROOT's device
// with a caller-supplied path filter and a cursor AGE seconds old; report event count,
// time to HistoryDone, and for the first three events only the first two path components
// (never a full private path).  pathprobe ROOT FILTER AGE_SECONDS FLAGS TIMEOUT
#include <CoreServices/CoreServices.h>
#include <dispatch/dispatch.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <time.h>
struct ctx { dispatch_semaphore_t done; size_t events; size_t shown; double first; double t0; };
static double now(void){ struct timespec t; clock_gettime(CLOCK_MONOTONIC,&t); return t.tv_sec+t.tv_nsec/1e9; }
static void cb(ConstFSEventStreamRef s, void *info, size_t n, void *paths, const FSEventStreamEventFlags f[], const FSEventStreamEventId ids[]) {
  struct ctx *c = info; char **p = paths; if (!c->first) c->first = now() - c->t0;
  for (size_t i=0;i<n;i++){
    c->events++;
    if (f[i] & kFSEventStreamEventFlagHistoryDone) { dispatch_semaphore_signal(c->done); continue; }
    if (c->shown < 3) { c->shown++;
      char buf[256]; strncpy(buf, p[i], 255); buf[255]=0; int slashes=0;
      for (char *q=buf; *q; q++){ if (*q=='/'){ slashes++; if (slashes==3){ *q=0; break; } } }
      fprintf(stderr, "  sample event: prefix=\"%s\" flags=0x%x\n", buf, (unsigned)f[i]); }
  }
}
int main(int argc, char **argv) {
  if (argc != 6) { fprintf(stderr,"usage: pathprobe ROOT FILTER AGE FLAGS TIMEOUT\n"); return 2; }
  struct stat st; struct statfs fs;
  if (stat(argv[1],&st) || statfs(argv[1],&fs)) return 2;
  long age = atol(argv[3]); unsigned flags = (unsigned)atol(argv[4]); long to = atol(argv[5]);
  FSEventStreamEventId cursor = FSEventsGetLastEventIdForDeviceBeforeTime(st.st_dev, (CFAbsoluteTime)(time(NULL) - age));
  CFStringRef p = CFStringCreateWithCString(NULL, argv[2], kCFStringEncodingUTF8);
  CFArrayRef ps = CFArrayCreate(NULL,(const void**)&p,1,&kCFTypeArrayCallBacks);
  struct ctx c = {.done = dispatch_semaphore_create(0)};
  FSEventStreamContext sc = {.info=&c};
  c.t0 = now();
  FSEventStreamRef s = FSEventStreamCreateRelativeToDevice(NULL, cb, &sc, st.st_dev, ps, cursor, 0.01, flags);
  dispatch_queue_t q = dispatch_queue_create("p", DISPATCH_QUEUE_SERIAL);
  FSEventStreamSetDispatchQueue(s, q);
  FSEventStreamStart(s);
  double ts = now();
  long r = dispatch_semaphore_wait(c.done, dispatch_time(DISPATCH_TIME_NOW, to*NSEC_PER_SEC));
  double th = now();
  FSEventStreamStop(s); FSEventStreamInvalidate(s); dispatch_sync(q, ^{}); FSEventStreamRelease(s);
  printf("{\"mnt\":\"%s\",\"filter\":\"%s\",\"age_s\":%ld,\"flags\":%u,\"cursor\":%llu,\"now_id\":%llu,\"first_cb_ms\":%.1f,\"history_ms\":%.1f,\"timeout\":%s,\"events\":%zu}\n",
    fs.f_mntonname, argv[2], age, flags, cursor, FSEventsGetCurrentEventId(), c.first*1e3, (th-ts)*1e3, r?"true":"false", c.events);
  return r ? 1 : 0;
}
