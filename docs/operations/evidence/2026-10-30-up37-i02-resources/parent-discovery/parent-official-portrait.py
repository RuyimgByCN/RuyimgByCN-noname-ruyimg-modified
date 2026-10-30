"""Inspect the official resource ZIP with bounded HTTP ranges; retain one named portrait."""
import hashlib
import io
import json
from pathlib import Path
import urllib.parse
import urllib.request
import zipfile
from PIL import Image

URL = 'https://github.com/libnoname/noname/releases/download/v1.11.6.1/noname.full.zip'
SIZE = 1216103840
EXPECTED_ARCHIVE_SHA256 = 'cb9fa579ae1abbd98d36208acc2937a8166d29466418a2d8ddf98d8118b376dc'
TARGETS = {'ol_jie_zhuran', 'ol_zhuhuan', 'ol_sunhanhua', 'ol_xushao', 'hm_yudu'}


class RangeFile:
    def __init__(self, total, fetch):
        self.total = total
        self.fetch = fetch
        self.pos = 0

    def tell(self):
        return self.pos

    def seekable(self):
        return True

    def seek(self, offset, whence=0):
        if whence not in (0, 1, 2):
            raise ValueError('Invalid seek origin')
        position = offset + (0 if whence == 0 else self.pos if whence == 1 else self.total)
        if not 0 <= position <= self.total:
            raise ValueError('Seek outside asset')
        self.pos = position
        return position

    def read(self, length=-1):
        length = self.total - self.pos if length < 0 else min(length, self.total - self.pos)
        if length > 8 * 1024 * 1024:
            raise ValueError('Reject large/full archive read')
        if not length:
            return b''
        data = self.fetch(self.pos, length)
        if len(data) != length:
            raise ValueError('Incomplete asset range')
        self.pos += len(data)
        return data


def self_check():
    data = io.BytesIO()
    with zipfile.ZipFile(data, 'w') as archive:
        archive.writestr('fixture.jpg', b'fixture')
    payload = data.getvalue()
    with zipfile.ZipFile(RangeFile(len(payload), lambda start, length: payload[start:start + length])) as archive:
        assert archive.read('fixture.jpg') == b'fixture'
    reader = RangeFile(SIZE, lambda start, length: b'')
    try:
        reader.read(SIZE)
    except ValueError:
        pass
    else:
        raise AssertionError('Full download must be refused')


def main():
    self_check()
    output = Path(__file__).with_name('parent-official-resources')
    output.mkdir(exist_ok=True)
    transferred = 0
    identity = None

    def fetch(start, length):
        nonlocal transferred, identity
        headers = {'Range': f'bytes={start}-{start + length - 1}', 'Accept-Encoding': 'identity',
                   'User-Agent': 'noname-resource-probe/1.0'}
        if identity:
            headers['If-Match'] = identity
        request = urllib.request.Request(URL, headers=headers)
        with urllib.request.urlopen(request, timeout=15) as response:
            if response.status != 206:
                raise ValueError(f'Range ignored (HTTP {response.status}); do not read body')
            if not response.headers.get('Content-Range', '').startswith(f'bytes {start}-'):
                raise ValueError('Unexpected Content-Range')
            host = urllib.parse.urlsplit(response.geturl()).hostname or ''
            if not (host == 'githubusercontent.com' or host.endswith('.githubusercontent.com')):
                raise ValueError('Unexpected final asset host')
            current = response.headers.get('ETag')
            if not current or (identity and current != identity):
                raise ValueError('Release identity absent or changed')
            identity = current
            data = response.read(length + 1)
            if len(data) != length:
                raise ValueError('Wrong range length')
            transferred += len(data)
            return data

    report = {'source': URL, 'release': 'v1.11.6.1', 'archive_expected_sha256': EXPECTED_ARCHIVE_SHA256,
              'full_archive_sha256_verified': False, 'reader_self_check': True, 'status': 'started'}
    result = output / 'provenance.json'
    result.write_text(json.dumps(report, indent=2) + '\n')
    try:
        with zipfile.ZipFile(RangeFile(SIZE, fetch)) as archive:
            entries = archive.infolist()
            matches = [entry for entry in entries if Path(entry.filename).stem in TARGETS
                       and Path(entry.filename).suffix.lower() in {'.jpg', '.jpeg', '.png', '.webp', '.avif'}]
            report['named_entries'] = [entry.filename for entry in matches]
            entry = archive.getinfo('image/character/hm_yudu.jpg')
            if not 0 < entry.file_size <= 4 * 1024 * 1024 or entry.compress_size > 4 * 1024 * 1024:
                raise ValueError('Unexpected portrait size')
            data = archive.read(entry)  # zipfile checks this entry's CRC; it does not verify the full ZIP digest.
            with Image.open(io.BytesIO(data)) as image:
                encoding = image.format
                dimensions = image.size
                if encoding not in {'JPEG', 'PNG', 'WEBP', 'GIF', 'AVIF'} or image.width * image.height > 16000000:
                    raise ValueError('Unexpected portrait encoding or dimensions')
                image.load()
            destination = output / 'hm_yudu.jpg'
            destination.write_bytes(data)
            report.update(status='complete', entry=entry.filename, bytes=len(data),
                          crc32=f'{entry.CRC:08x}', entry_crc_verified=True,
                          portrait_sha256=hashlib.sha256(data).hexdigest(), local_file=destination.name,
                          actual_encoding=encoding, dimensions=dimensions, pixels_decoded=True,
                          scope='Official resource release supplemental image only; adopted gameplay source unchanged')
    except Exception as error:
        report.update(status='failed', error=str(error))
    finally:
        report.update(bytes_transferred=transferred, asset_etag=identity)
        result.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    if report['status'] != 'complete':
        raise SystemExit(1)


if __name__ == '__main__':
    main()
