"""Inspect x64 PE imports so packages cannot silently require a developer CRT."""
import struct


def imported_libraries(data):
    def unpack(format, offset):
        if offset < 0 or offset + struct.calcsize(format) > len(data):
            raise ValueError('Truncated Windows executable.')
        return struct.unpack_from(format, data, offset)

    if data[:2] != b'MZ':
        raise ValueError('Expected a Windows executable.')
    pe, = unpack('<I', 0x3c)
    if data[pe:pe+4] != b'PE\0\0':
        raise ValueError('Invalid Windows executable signature.')
    machine, section_count = unpack('<HH', pe+4)
    optional_size, = unpack('<H', pe+20)
    optional = pe+24
    magic, = unpack('<H', optional)
    if machine != 0x8664 or magic != 0x20b or optional_size < 128:
        raise ValueError('Expected a Windows x64 executable.')
    imports, size = unpack('<II', optional+120)
    if imports == 0 and size == 0:
        return []
    sections = []
    for index in range(section_count):
        _, address, raw_size, raw_offset = unpack('<IIII', optional+optional_size+index*40+8)
        sections.append((address, raw_size, raw_offset))

    def offset(rva, length):
        for address, raw_size, raw_offset in sections:
            if address <= rva and rva+length <= address+raw_size:
                location = raw_offset+rva-address
                if location+length <= len(data):
                    return location
        raise ValueError('Invalid Windows import address.')

    libraries = []
    for position in range(0, size-19, 20):
        descriptor = unpack('<IIIII', offset(imports+position, 20))
        if not any(descriptor):
            return libraries
        name = bytearray()
        for index in range(260):
            character = data[offset(descriptor[3]+index, 1)]
            if character == 0:
                if not name:
                    raise ValueError('Empty Windows import name.')
                libraries.append(name.decode('ascii').lower())
                break
            name.append(character)
        else:
            raise ValueError('Invalid Windows import name.')
    raise ValueError('Unterminated Windows import directory.')


def verify_static_crt(path):
    libraries = imported_libraries(path.read_bytes())
    extra = [name for name in libraries if name.startswith(('vcruntime', 'msvcp', 'msvcr', 'concrt'))]
    if extra:
        raise ValueError('Windows package requires unbundled Visual C++ runtime: '+', '.join(extra))
    return libraries
