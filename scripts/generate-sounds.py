"""Generate original, deterministic RVNFlow sounds (MIT, no sampled recordings)."""
import math
import pathlib
import struct
import wave

DEST = pathlib.Path(__file__).resolve().parents[1] / 'src-tauri' / 'sounds'
DEST.mkdir(exist_ok=True)
RATE = 44100
SOUNDS = {
    'chime': (1.6, [(523.25, 0), (659.25, .18), (783.99, .36)]),
    'bell': (1.7, [(880, 0), (1760, 0), (2637, 0)]),
    'digital': (.65, [(740, 0), (988, .2)]),
    'mechanical': (.045, [(1900, 0), (3400, 0)]),
    'soft': (.055, [(900, 0)]),
}
for name, (duration, tones) in SOUNDS.items():
    data = bytearray()
    for index in range(int(duration * RATE)):
        t = index / RATE
        value = 0
        for frequency, start in tones:
            age = t - start
            if age >= 0:
                decay = 90 if duration < .1 else 4
                envelope = min(1, age * 1000) * math.exp(-decay * age)
                value += math.sin(2 * math.pi * frequency * age) * envelope
        value *= min(1, (duration - t) * 100) * .55 / len(tones)
        data.extend(struct.pack('<h', int(value * 32767)))
    with wave.open(str(DEST / f'{name}.wav'), 'wb') as output:
        output.setparams((1, 2, RATE, 0, 'NONE', 'not compressed'))
        output.writeframes(data)
