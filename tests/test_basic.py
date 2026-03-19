import pytest
import png_parser
from PIL import Image
import os

@pytest.fixture
def carrier(tmp_path):
    p = tmp_path / "carrier.png"
    Image.new("RGB", (500, 500), color=(100, 149, 237)).save(str(p))
    return str(p)

@pytest.fixture
def secret(tmp_path):
    p = tmp_path / "secret.txt"
    p.write_text("test payload content")
    return str(p)

def test_hide_reveal_plain(carrier, secret, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret)
    result = png_parser.reveal(out, str(tmp_path))
    assert os.path.exists(result)

def test_hide_reveal_encrypted(carrier, secret, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret, password="abc123")
    result = png_parser.reveal(out, str(tmp_path), password="abc123")
    assert os.path.exists(result)

def test_wrong_password(carrier, secret, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret, password="correct")
    with pytest.raises(Exception):
        png_parser.reveal(out, str(tmp_path), password="wrong")

def test_verify(carrier, secret, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret, password="abc123")
    assert png_parser.verify(out, "abc123") == True
    assert png_parser.verify(out, "wrong") == False

def test_expiry(carrier, secret, tmp_path):
    import time
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret,
        password="abc", expires_seconds=1)
    time.sleep(2)
    with pytest.raises(Exception):
        png_parser.reveal(out, str(tmp_path), password="abc")

def test_delete(carrier, secret, tmp_path):
    out     = str(tmp_path / "out.png")
    cleaned = str(tmp_path / "clean.png")
    png_parser.hide(carrier, out, secret, password="abc")
    png_parser.delete(out, cleaned, password="abc")
    with pytest.raises(Exception):
        png_parser.reveal(cleaned, str(tmp_path), password="abc")

def test_capacity(carrier):
    assert png_parser.capacity(carrier, "chunk") > 0
    assert png_parser.capacity(carrier, "pixel") > 0