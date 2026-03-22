import pytest
import json
import os
import time
import shutil
import png_parser

IMAGES = os.path.join(os.path.dirname(__file__), "images")


# ── Fixtures ──────────────────────────────────────────────────

@pytest.fixture
def carrier():
    return os.path.join(IMAGES, "carrier_large.png")

@pytest.fixture
def small_carrier():
    return os.path.join(IMAGES, "carrier_small.png")

@pytest.fixture
def carrier_bmp():
    return os.path.join(IMAGES, "carrier_bmp.bmp")

@pytest.fixture
def carrier_tiff():
    return os.path.join(IMAGES, "carrier_tiff.tiff")

@pytest.fixture
def three_carriers():
    return [
        os.path.join(IMAGES, "carrier_large.png"),
        os.path.join(IMAGES, "carrier_bmp.bmp"),
        os.path.join(IMAGES, "carrier_tiff.tiff"),
    ]

@pytest.fixture
def secret_txt(tmp_path):
    p = tmp_path / "secret.txt"
    p.write_text("hello from png_parser v0.3.0")
    return str(p)

@pytest.fixture
def secret_bin(tmp_path):
    p = tmp_path / "secret.bin"
    p.write_bytes(os.urandom(1024))
    return str(p)

@pytest.fixture
def secret_large(tmp_path):
    p = tmp_path / "large.bin"
    p.write_bytes(os.urandom(1024 * 100))
    return str(p)


# ── Version ───────────────────────────────────────────────────

def test_version():
    assert hasattr(png_parser, "__version__")
    assert png_parser.__version__ == "0.3.0"


# ── hide + reveal — plain ─────────────────────────────────────

def test_hide_reveal_plain(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt)
    result = png_parser.reveal(out, str(tmp_path))
    assert os.path.exists(result)
    with open(result) as f:
        assert f.read() == "hello from png_parser v0.3.0"

def test_hide_reveal_binary(carrier, secret_bin, tmp_path):
    original = open(secret_bin, "rb").read()
    out      = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_bin)
    result = png_parser.reveal(out, str(tmp_path))
    assert open(result, "rb").read() == original

def test_hide_reveal_large(carrier, secret_large, tmp_path):
    original = open(secret_large, "rb").read()
    out      = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_large)
    result = png_parser.reveal(out, str(tmp_path))
    assert open(result, "rb").read() == original

def test_original_filename_preserved(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    outdir = str(tmp_path / "extracted")
    os.makedirs(outdir)
    png_parser.hide(carrier, out, secret_txt)
    result = png_parser.reveal(out, outdir)
    assert os.path.basename(result) == "secret.txt"


# ── hide + reveal — encrypted ─────────────────────────────────

def test_hide_reveal_encrypted(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="abc123")
    result = png_parser.reveal(out, str(tmp_path), password="abc123")
    assert os.path.exists(result)
    with open(result) as f:
        assert f.read() == "hello from png_parser v0.3.0"

def test_wrong_password_raises(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="correct")
    with pytest.raises(Exception):
        png_parser.reveal(out, str(tmp_path), password="wrong")

def test_missing_password_raises(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="secret")
    with pytest.raises(Exception):
        png_parser.reveal(out, str(tmp_path))


# ── Expiry ────────────────────────────────────────────────────

def test_expiry_seconds(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt,
        password="x", expires_seconds=1)
    time.sleep(2)
    with pytest.raises(Exception):
        png_parser.reveal(out, str(tmp_path), password="x")

def test_expiry_not_yet_expired(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt,
        password="x", expires_seconds=60)
    result = png_parser.reveal(out, str(tmp_path), password="x")
    assert os.path.exists(result)

def test_expiry_additive(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt,
        password="x",
        expires_days=0,
        expires_hours=0,
        expires_minutes=1,
        expires_seconds=0)
    result = png_parser.reveal(out, str(tmp_path), password="x")
    assert os.path.exists(result)

def test_permanent_never_expires(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="x")
    result = png_parser.reveal(out, str(tmp_path), password="x")
    assert os.path.exists(result)


# ── verify ────────────────────────────────────────────────────

def test_verify_correct_password(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="abc123")
    assert png_parser.verify(out, "abc123") is True

def test_verify_wrong_password(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="abc123")
    assert png_parser.verify(out, "wrongpass") is False


# ── info ──────────────────────────────────────────────────────

def test_info_encrypted_correct_password(carrier, secret_txt, tmp_path):
    out  = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="abc123")
    result = png_parser.info(out, password="abc123")
    assert "encrypted   : true"    in result
    assert "filename    : secret.txt" in result
    assert "file_size"             in result
    assert "fingerprint"           in result

def test_info_encrypted_no_password(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="abc123")
    result = png_parser.info(out)
    assert "encrypted   : true" in result
    assert "fingerprint"        in result
    assert "filename"           not in result.split("fingerprint")[0]

def test_info_plain(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt)
    result = png_parser.info(out)
    assert "encrypted   : false"     in result
    assert "filename    : secret.txt" in result

def test_info_permanent(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="x")
    result = png_parser.info(out, password="x")
    assert "expires_at  : permanent" in result

def test_info_with_expiry(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt,
        password="x", expires_hours=1)
    result = png_parser.info(out, password="x")
    assert "unix:" in result

# ── delete ────────────────────────────────────────────────────

def test_delete_encrypted(carrier, secret_txt, tmp_path):
    out     = str(tmp_path / "out.png")
    cleaned = str(tmp_path / "clean.png")
    png_parser.hide(carrier, out, secret_txt, password="abc123")
    png_parser.delete(out, cleaned, password="abc123")
    with pytest.raises(Exception):
        png_parser.reveal(cleaned, str(tmp_path), password="abc123")

def test_delete_plain(carrier, secret_txt, tmp_path):
    out     = str(tmp_path / "out.png")
    cleaned = str(tmp_path / "clean.png")
    png_parser.hide(carrier, out, secret_txt)
    png_parser.delete(out, cleaned)
    with pytest.raises(Exception):
        png_parser.reveal(cleaned, str(tmp_path))

def test_delete_wrong_password_raises(carrier, secret_txt, tmp_path):
    out     = str(tmp_path / "out.png")
    cleaned = str(tmp_path / "clean.png")
    png_parser.hide(carrier, out, secret_txt, password="correct")
    with pytest.raises(Exception):
        png_parser.delete(out, cleaned, password="wrong")

def test_delete_no_password_raises(carrier, secret_txt, tmp_path):
    out     = str(tmp_path / "out.png")
    cleaned = str(tmp_path / "clean.png")
    png_parser.hide(carrier, out, secret_txt, password="secret")
    with pytest.raises(Exception):
        png_parser.delete(out, cleaned)


# ── reencrypt ─────────────────────────────────────────────────

def test_reencrypt(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    newout = str(tmp_path / "new.png")
    png_parser.hide(carrier, out, secret_txt, password="old")
    png_parser.reencrypt(out, newout, "old", "new")
    result = png_parser.reveal(newout, str(tmp_path), password="new")
    assert os.path.exists(result)

def test_reencrypt_old_password_fails(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    newout = str(tmp_path / "new.png")
    png_parser.hide(carrier, out, secret_txt, password="old")
    png_parser.reencrypt(out, newout, "old", "new")
    with pytest.raises(Exception):
        png_parser.reveal(newout, str(tmp_path), password="old")

def test_reencrypt_wrong_old_password_raises(carrier, secret_txt, tmp_path):
    out    = str(tmp_path / "out.png")
    newout = str(tmp_path / "new.png")
    png_parser.hide(carrier, out, secret_txt, password="correct")
    with pytest.raises(Exception):
        png_parser.reencrypt(out, newout, "wrong", "new")


# ── capacity ──────────────────────────────────────────────────

def test_capacity_chunk(carrier):
    assert png_parser.capacity(carrier, "chunk") > 0

def test_capacity_pixel(carrier):
    assert png_parser.capacity(carrier, "pixel") > 0

def test_capacity_chunk_greater_than_pixel(carrier):
    assert png_parser.capacity(carrier, "chunk") > \
           png_parser.capacity(carrier, "pixel")

def test_capacity_invalid_mode_raises(carrier):
    with pytest.raises(Exception):
        png_parser.capacity(carrier, "invalid")


# ── fingerprint ───────────────────────────────────────────────

def test_fingerprint_is_hex(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt, password="x")
    fp  = png_parser.fingerprint(out)
    assert len(fp) == 64
    assert all(c in "0123456789abcdef" for c in fp)

def test_fingerprint_same_image_same_fp(carrier, secret_txt, tmp_path):
    out1 = str(tmp_path / "out1.png")
    out2 = str(tmp_path / "out2.png")
    png_parser.hide(carrier, out1, secret_txt, password="x")
    shutil.copy(out1, out2)
    assert png_parser.fingerprint(out1) == png_parser.fingerprint(out2)

def test_fingerprint_different_password_different_fp(carrier, secret_txt, tmp_path):
    out1 = str(tmp_path / "out1.png")
    out2 = str(tmp_path / "out2.png")
    png_parser.hide(carrier, out1, secret_txt, password="x")
    png_parser.hide(carrier, out2, secret_txt, password="y")
    assert png_parser.fingerprint(out1) != png_parser.fingerprint(out2)


# ── split + merge ─────────────────────────────────────────────

def test_split_merge_roundtrip(three_carriers, secret_large, tmp_path):
    original   = open(secret_large, "rb").read()
    shards_dir = str(tmp_path / "shards")
    output_dir = str(tmp_path / "output")
    os.makedirs(shards_dir)
    os.makedirs(output_dir)

    shards = png_parser.split(
        secret_large, three_carriers, shards_dir, password="x")
    assert len(shards) == 3

    result = png_parser.merge(shards, output_dir, password="x")
    assert open(result, "rb").read() == original

def test_merge_any_order(three_carriers, secret_large, tmp_path):
    original   = open(secret_large, "rb").read()
    shards_dir = str(tmp_path / "shards")
    output_dir = str(tmp_path / "output")
    os.makedirs(shards_dir)
    os.makedirs(output_dir)

    shards = png_parser.split(
        secret_large, three_carriers, shards_dir, password="x")
    result = png_parser.merge(list(reversed(shards)), output_dir, password="x")
    assert open(result, "rb").read() == original

def test_merge_wrong_password_raises(three_carriers, secret_large, tmp_path):
    shards_dir = str(tmp_path / "shards")
    output_dir = str(tmp_path / "output")
    os.makedirs(shards_dir)
    os.makedirs(output_dir)

    shards = png_parser.split(
        secret_large, three_carriers, shards_dir, password="correct")
    with pytest.raises(Exception):
        png_parser.merge(shards, output_dir, password="wrong")

def test_split_merge_with_expiry(three_carriers, secret_txt, tmp_path):
    shards_dir = str(tmp_path / "shards")
    output_dir = str(tmp_path / "output")
    os.makedirs(shards_dir)
    os.makedirs(output_dir)

    shards = png_parser.split(
        secret_txt, three_carriers, shards_dir,
        password="x", expires_seconds=1)
    time.sleep(2)
    with pytest.raises(Exception):
        png_parser.merge(shards, output_dir, password="x")


# ── pixel mode ────────────────────────────────────────────────

def test_hide_reveal_pixel_mode(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.png")
    png_parser.hide(carrier, out, secret_txt,
        password="x", mode_str="pixel")
    result = png_parser.reveal(out, str(tmp_path), password="x")
    with open(result) as f:
        assert f.read() == "hello from png_parser v0.3.0"
        
def test_pixel_mode_small_image_raises(small_carrier, secret_large, tmp_path):
    out = str(tmp_path / "out.png")
    with pytest.raises(Exception):
        png_parser.hide(small_carrier, out, secret_large,
            password="x", mode="pixel")


# ── BMP format ────────────────────────────────────────────────

def test_hide_reveal_bmp(carrier_bmp, secret_txt, tmp_path):
    out = str(tmp_path / "out.bmp")
    png_parser.hide(carrier_bmp, out, secret_txt, password="x")
    result = png_parser.reveal(out, str(tmp_path), password="x")
    with open(result) as f:
        assert f.read() == "hello from png_parser v0.3.0"


# ── TIFF format ───────────────────────────────────────────────

def test_hide_reveal_tiff(carrier_tiff, secret_txt, tmp_path):
    out = str(tmp_path / "out.tiff")
    png_parser.hide(carrier_tiff, out, secret_txt, password="x")
    result = png_parser.reveal(out, str(tmp_path), password="x")
    with open(result) as f:
        assert f.read() == "hello from png_parser v0.3.0"


# ── Format rejection ──────────────────────────────────────────

def test_jpeg_output_rejected(carrier, secret_txt, tmp_path):
    out = str(tmp_path / "out.jpg")
    with pytest.raises(Exception):
        png_parser.hide(carrier, out, secret_txt)

def test_no_payload_raises(carrier, tmp_path):
    with pytest.raises(Exception):
        png_parser.reveal(carrier, str(tmp_path))

def test_invalid_file_raises(tmp_path, secret_txt):
    bad = str(tmp_path / "bad.png")
    with open(bad, "w") as f:
        f.write("this is not an image")
    with pytest.raises(Exception):
        png_parser.reveal(bad, str(tmp_path))