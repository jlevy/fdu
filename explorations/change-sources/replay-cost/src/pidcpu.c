// Test: can a non-root process read fseventsd's CPU time via proc_pidinfo?
#include <libproc.h>
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <string.h>
int main(int argc, char **argv) {
  if (argc != 2) return 2;
  pid_t pid = atoi(argv[1]);
  struct proc_taskinfo ti;
  int n = proc_pidinfo(pid, PROC_PIDTASKINFO, 0, &ti, sizeof ti);
  if (n != (int)sizeof ti) { printf("{\"ok\":false,\"n\":%d,\"errno\":%d,\"msg\":\"%s\"}\n", n, errno, strerror(errno)); return 1; }
  printf("{\"ok\":true,\"user_ns\":%llu,\"system_ns\":%llu,\"total_s\":%.4f,\"threads\":%d,\"csw\":%d,\"syscalls_mach\":%d,\"syscalls_unix\":%d,\"faults\":%d,\"pageins\":%d}\n",
    ti.pti_total_user, ti.pti_total_system, (ti.pti_total_user+ti.pti_total_system)/1e9, ti.pti_threadnum, ti.pti_csw, ti.pti_syscalls_mach, ti.pti_syscalls_unix, ti.pti_faults, ti.pti_pageins);
  struct rusage_info_v4 ru;
  int r = proc_pid_rusage(pid, RUSAGE_INFO_V4, (rusage_info_t*)&ru);
  if (r == 0) printf("{\"rusage_ok\":true,\"user_ns\":%llu,\"system_ns\":%llu,\"diskio_bytesread\":%llu,\"diskio_byteswritten\":%llu,\"logical_writes\":%llu,\"cpu_instructions\":%llu,\"cycles\":%llu}\n",
    ru.ri_user_time, ru.ri_system_time, ru.ri_diskio_bytesread, ru.ri_diskio_byteswritten, ru.ri_logical_writes, ru.ri_instructions, ru.ri_cycles);
  else printf("{\"rusage_ok\":false,\"errno\":%d,\"msg\":\"%s\"}\n", errno, strerror(errno));
  return 0;
}
