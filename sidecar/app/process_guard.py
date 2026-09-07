from __future__ import annotations

import ctypes
import os
import threading
import time

PARENT_PID_ENV = "TERMINAL_MATE_PARENT_PID"


def start_parent_process_guard() -> bool:
    raw_pid = os.getenv(PARENT_PID_ENV, "").strip()
    if not raw_pid:
        return False

    try:
        parent_pid = int(raw_pid)
    except ValueError:
        return False

    if parent_pid <= 0:
        return False

    thread = threading.Thread(
        target=_monitor_parent,
        args=(parent_pid,),
        name="terminal-mate-parent-guard",
        daemon=True,
    )
    thread.start()
    return True


def _monitor_parent(parent_pid: int) -> None:
    while _process_is_running(parent_pid):
        time.sleep(1)

    os._exit(0)


def _process_is_running(pid: int) -> bool:
    if os.name == "nt":
        return _windows_process_is_running(pid)

    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def _windows_process_is_running(pid: int) -> bool:
    process_query_limited_information = 0x1000
    still_active = 259
    kernel32 = ctypes.windll.kernel32
    handle = kernel32.OpenProcess(process_query_limited_information, False, pid)
    if not handle:
        return False

    try:
        exit_code = ctypes.c_ulong()
        if not kernel32.GetExitCodeProcess(handle, ctypes.byref(exit_code)):
            return False
        return exit_code.value == still_active
    finally:
        kernel32.CloseHandle(handle)
