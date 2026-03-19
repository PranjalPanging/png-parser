from typing import Optional

def hide(
    input_path:      str,
    output_path:     str,
    file_path:       str,
    password:        Optional[str] = None,
    mode_str:        str = "chunk",
    expires_days:    Optional[int] = None,
    expires_hours:   Optional[int] = None,
    expires_minutes: Optional[int] = None,
    expires_seconds: Optional[int] = None,
) -> None: ...

def reveal(
    input_path:  str,
    output_path: str,
    password:    Optional[str] = None,
) -> str: ...

def info(
    input_path: str,
    password:   Optional[str] = None,
) -> str: ...

def verify(
    input_path: str,
    password:   str,
) -> bool: ...

def delete(
    input_path:  str,
    output_path: str,
    password:    Optional[str] = None,
) -> None: ...

def reencrypt(
    input_path:   str,
    output_path:  str,
    old_password: str,
    new_password: str,
) -> None: ...

def capacity(
    input_path: str,
    mode_str:   str = "chunk",
) -> int: ...

def fingerprint(
    input_path: str,
) -> str: ...

def split(
    file_path:       str,
    carriers:        list[str],
    output_dir:      str,
    password:        Optional[str] = None,
    expires_days:    Optional[int] = None,
    expires_hours:   Optional[int] = None,
    expires_minutes: Optional[int] = None,
    expires_seconds: Optional[int] = None,
) -> list[str]: ...

def merge(
    inputs:      list[str],
    output_path: str,
    password:    Optional[str] = None,
) -> str: ...

__version__: str

