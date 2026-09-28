// Live FSEvents watcher for the writer-coverage experiments (review instrument, no fdu linkage).
// usage: livewatch ROOT [LATENCY_SECONDS] [CREATE_FLAGS]
// Starts a stream at kFSEventStreamEventIdSinceNow, prints one JSON object per line, and
// accepts commands on stdin: "mark LABEL", "flush", "quit". Every printed line carries a
// monotonic timestamp so the driver can align events with the operations it performed.
#include <CoreServices/CoreServices.h>
#include <dispatch/dispatch.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

struct context {
    const char *root;
    size_t root_len;
    size_t count;
    size_t callbacks;
};

static double monotonic(void) {
    struct timespec value;
    clock_gettime(CLOCK_MONOTONIC, &value);
    return (double)value.tv_sec + (double)value.tv_nsec / 1e9;
}

static void json_string(const char *s) {
    putchar('"');
    for (const unsigned char *p = (const unsigned char *)s; *p; ++p) {
        if (*p == '"' || *p == '\\') { putchar('\\'); putchar(*p); }
        else if (*p < 0x20) printf("\\u%04x", *p);
        else putchar(*p);
    }
    putchar('"');
}

static void callback(ConstFSEventStreamRef stream, void *info, size_t count, void *paths,
                     const FSEventStreamEventFlags flags[], const FSEventStreamEventId ids[]) {
    (void)stream;
    struct context *ctx = info;
    double t = monotonic();
    ++ctx->callbacks;
    const char **names = paths;
    for (size_t i = 0; i < count; ++i) {
        const char *name = names[i];
        const char *rel = name;
        if (strncmp(name, ctx->root, ctx->root_len) == 0 &&
            (name[ctx->root_len] == '/' || name[ctx->root_len] == '\0')) {
            rel = name + ctx->root_len;
            while (*rel == '/') ++rel;
        }
        printf("{\"type\":\"event\",\"t\":%.6f,\"id\":%" PRIu64 ",\"flags\":%u,\"callback\":%zu,\"rel\":",
               t, ids[i], flags[i], ctx->callbacks);
        json_string(rel);
        printf(",\"inside\":%s}\n", rel != name ? "true" : "false");
        ++ctx->count;
    }
    fflush(stdout);
}

int main(int argc, char **argv) {
    if (argc < 2 || argc > 4) {
        fprintf(stderr, "usage: livewatch ROOT [LATENCY_SECONDS] [CREATE_FLAGS]\n");
        return 2;
    }
    setvbuf(stdout, NULL, _IOLBF, 0);
    char root[PATH_MAX];
    struct stat st;
    struct statfs fs;
    if (!realpath(argv[1], root) || stat(root, &st) || statfs(root, &fs) || !S_ISDIR(st.st_mode)) {
        perror("root");
        return 2;
    }
    double latency = argc >= 3 ? atof(argv[2]) : 0.0;
    FSEventStreamCreateFlags create_flags = argc >= 4 ? (FSEventStreamCreateFlags)strtoul(argv[3], NULL, 10)
                                                      : (kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagNoDefer);
    struct context ctx = {.root = root, .root_len = strlen(root)};
    CFStringRef path = CFStringCreateWithCString(NULL, root, kCFStringEncodingUTF8);
    CFArrayRef paths = CFArrayCreate(NULL, (const void **)&path, 1, &kCFTypeArrayCallBacks);
    FSEventStreamContext sc = {.info = &ctx};
    FSEventStreamRef stream = FSEventStreamCreate(NULL, callback, &sc, paths, kFSEventStreamEventIdSinceNow,
                                                  latency, create_flags);
    CFRelease(paths);
    CFRelease(path);
    if (!stream) { fprintf(stderr, "stream create failed\n"); return 2; }
    dispatch_queue_t queue = dispatch_queue_create("review.livewatch", DISPATCH_QUEUE_SERIAL);
    FSEventStreamSetDispatchQueue(stream, queue);
    double t0 = monotonic();
    if (!FSEventStreamStart(stream)) { fprintf(stderr, "stream start failed\n"); return 2; }
    // Device fence and global counter at start, for the short-cursor replay comparison.
    FSEventStreamEventId fence = FSEventsGetLastEventIdForDeviceBeforeTime(st.st_dev, (CFAbsoluteTime)time(NULL));
    FSEventStreamEventId now_id = FSEventsGetCurrentEventId();
    FSEventStreamEventId stream_latest = FSEventStreamGetLatestEventId(stream);
    dispatch_sync(queue, ^{
        printf("{\"type\":\"ready\",\"t\":%.6f,\"fs\":\"%s\",\"device\":%d,\"device_fence\":%" PRIu64
               ",\"global_now\":%" PRIu64 ",\"stream_latest\":%" PRIu64 ",\"latency\":%.3f,\"create_flags\":%u,\"pid\":%d}\n",
               t0, fs.f_fstypename, (int)st.st_dev, fence, now_id, stream_latest, latency, create_flags, getpid());
        fflush(stdout);
    });
    char line[4096];
    while (fgets(line, sizeof line, stdin)) {
        size_t n = strlen(line);
        while (n && (line[n - 1] == '\n' || line[n - 1] == '\r')) line[--n] = 0;
        if (!strcmp(line, "quit")) break;
        if (!strcmp(line, "flush")) {
            double a = monotonic();
            FSEventStreamFlushSync(stream);
            double b = monotonic();
            FSEventStreamEventId latest = FSEventStreamGetLatestEventId(stream);
            dispatch_sync(queue, ^{
                printf("{\"type\":\"flushed\",\"t\":%.6f,\"flush_ms\":%.3f,\"stream_latest\":%" PRIu64
                       ",\"global_now\":%" PRIu64 ",\"events_so_far\":%zu}\n",
                       b, (b - a) * 1000, latest, FSEventsGetCurrentEventId(), ctx.count);
                fflush(stdout);
            });
            continue;
        }
        if (!strncmp(line, "mark ", 5)) {
            const char *label = line + 5;
            double t = monotonic();
            dispatch_sync(queue, ^{
                printf("{\"type\":\"mark\",\"t\":%.6f,\"label\":", t);
                json_string(label);
                printf(",\"global_now\":%" PRIu64 ",\"events_so_far\":%zu}\n", FSEventsGetCurrentEventId(), ctx.count);
                fflush(stdout);
            });
            continue;
        }
        fprintf(stderr, "unknown command: %s\n", line);
    }
    FSEventStreamFlushSync(stream);
    FSEventStreamStop(stream);
    FSEventStreamInvalidate(stream);
    dispatch_sync(queue, ^{});
    FSEventStreamEventId latest = FSEventStreamGetLatestEventId(stream);
    FSEventStreamRelease(stream);
    printf("{\"type\":\"summary\",\"t\":%.6f,\"events\":%zu,\"callbacks\":%zu,\"stream_latest\":%" PRIu64
           ",\"global_now\":%" PRIu64 "}\n", monotonic(), ctx.count, ctx.callbacks, latest, FSEventsGetCurrentEventId());
    dispatch_release(queue);
    return 0;
}
