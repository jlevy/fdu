// dirstat_fsctl: read APFS directory stats through the same fsctl apfs.util -S uses
// (APFSIOC_GET_DIR_STATS_EXT, _IOWR('J', 113, 272-byte struct), private API; GET only —
// this program never sets the maintain flag). Prints the raw struct so field offsets can
// be verified against apfs.util -S output, then times repeated calls.
// Usage: dirstat_fsctl DIR [REPEAT]
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mount.h>
#include <time.h>
#include <unistd.h>

#define APFSIOC_GET_DIR_STATS_EXT 0xC1104A71u

static double now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1e6;
}

int main(int argc, char **argv) {
    if (argc < 2) { fprintf(stderr, "usage: %s DIR [REPEAT]\n", argv[0]); return 2; }
    int repeat = argc > 2 ? atoi(argv[2]) : 0;
    unsigned char buf[272];
    memset(buf, 0, sizeof buf);
    double t0 = now_ms();
    int rc = fsctl(argv[1], APFSIOC_GET_DIR_STATS_EXT, buf, 0);
    double t1 = now_ms();
    if (rc != 0) { printf("%s: fsctl failed: %s (errno %d)\n", argv[1], strerror(errno), errno); return 1; }
    printf("%s: first call %.3f ms\n", argv[1], t1 - t0);
    for (int i = 0; i < 272; i += 8) {
        uint64_t v; memcpy(&v, buf + i, 8);
        if (v) printf("  +0x%02x: %" PRIu64 " (0x%" PRIx64 ")\n", i, v, v);
    }
    uint64_t gen, desc, phys;
    memcpy(&gen, buf + 0x30, 8); memcpy(&desc, buf + 0x38, 8); memcpy(&phys, buf + 0x40, 8);
    printf("  gen_count=%" PRIu64 " descendants=%" PRIu64 " physical_size=%" PRIu64 "\n", gen, desc, phys);
    if (repeat > 0) {
        double best = 1e9, sum = 0;
        for (int i = 0; i < repeat; i++) {
            memset(buf, 0, sizeof buf);
            double a = now_ms();
            if (fsctl(argv[1], APFSIOC_GET_DIR_STATS_EXT, buf, 0) != 0) { perror("fsctl"); return 1; }
            double d = now_ms() - a;
            sum += d; if (d < best) best = d;
        }
        printf("  repeat=%d mean=%.4f ms min=%.4f ms\n", repeat, sum / repeat, best);
    }
    return 0;
}
