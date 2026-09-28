// lockwait: take the shared timing lock ($REVIEW/timing.lock) if it becomes free within
// TIMEOUT seconds, then exec the command either way; report which on stderr so the run
// log records whether the measurement was serialized. The lock fd is close-on-exec-free
// on purpose: it stays held by the exec'd command and is released when it exits.
// usage: lockwait LOCKFILE TIMEOUT_SECONDS command...
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/file.h>
#include <time.h>
#include <unistd.h>

int main(int argc, char **argv) {
    if (argc < 4) { fprintf(stderr, "usage: lockwait LOCKFILE TIMEOUT command...\n"); return 2; }
    int fd = open(argv[1], O_RDWR | O_CREAT, 0644);
    if (fd < 0) { perror("open lock"); return 1; }
    int timeout = atoi(argv[2]);
    time_t start = time(NULL);
    int held = 0;
    for (;;) {
        if (flock(fd, LOCK_EX | LOCK_NB) == 0) { held = 1; break; }
        if (errno != EWOULDBLOCK) { perror("flock"); break; }
        if (time(NULL) - start >= timeout) break;
        usleep(250000);
    }
    fprintf(stderr, "[lockwait] lock %s after %ld s\n", held ? "held" : "NOT held (timeout)", (long)(time(NULL) - start));
    if (!held) close(fd);
    execvp(argv[3], argv + 3);
    perror("exec");
    return 127;
}
