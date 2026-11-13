"""Render original exploration loops without using game audio or samples."""

from array import array
from math import pi, sin
from pathlib import Path
import sys
import wave

RATE = 32_000

# Each bar has eight eighth notes. Zero is a rest.
PALLET = (
    (69, 0, 73, 76, 78, 76, 73, 0),
    (71, 0, 74, 76, 81, 78, 76, 0),
    (73, 76, 81, 0, 83, 81, 76, 73),
    (71, 0, 76, 74, 73, 71, 69, 0),
    (73, 0, 76, 78, 81, 0, 78, 76),
    (74, 0, 78, 81, 83, 81, 78, 0),
    (73, 76, 74, 73, 71, 0, 76, 0),
    (74, 73, 71, 0, 68, 71, 76, 0),
)
ROUTE = (
    (67, 69, 71, 0, 74, 71, 69, 74),
    (76, 74, 72, 0, 69, 72, 76, 79),
    (78, 76, 74, 71, 0, 74, 76, 78),
    (76, 71, 67, 69, 71, 74, 71, 0),
    (79, 76, 74, 0, 76, 78, 79, 81),
    (83, 79, 76, 74, 72, 76, 79, 0),
    (78, 76, 74, 71, 72, 69, 71, 74),
    (76, 74, 71, 0, 69, 66, 69, 0),
)


def render(path, melody, chords, tempo):
    eighth = round(RATE * 30 / tempo)
    bar_length = 8 * eighth
    left = [0.0] * (len(melody) * bar_length)
    right = left.copy()

    def note(start, length, pitch, volume, pan, brightness):
        frequency = 440 * 2 ** ((pitch - 69) / 12)
        phase_step = 2 * pi * frequency / RATE
        attack = round(RATE * 0.012)
        release = round(RATE * 0.055)
        for sample in range(length):
            envelope = min(1, sample / attack, (length - 1 - sample) / release)
            phase = sample * phase_step
            tone = sin(phase) + brightness * (0.7 * sin(2 * phase) + 0.3 * sin(3 * phase))
            value = volume * tone * envelope * (1 - 0.4 * sample / length)
            left[start + sample] += value * (1 - pan)
            right[start + sample] += value * (1 + pan)

    for bar, (notes, chord) in enumerate(zip(melody, chords, strict=True)):
        start = bar * bar_length
        for beat, pitch in enumerate(notes):
            if pitch:
                note(start + beat * eighth, eighth, pitch, 0.15, 0.13, 0.40)
            note(start + beat * eighth, eighth, chord[(0, 1, 2, 1)[beat % 4]], 0.058, -0.28, 0.25)
        for beat in (0, 4):
            note(start + beat * eighth, 3 * eighth, chord[0] - 12, 0.105, 0, 0.15)

    pcm = array("h", (round(sample * 32767) for frame in zip(left, right) for sample in frame))
    if sys.byteorder != "little":
        pcm.byteswap()
    path.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(path), "wb") as wav:
        wav.setparams((2, 2, RATE, 0, "NONE", "not compressed"))
        wav.writeframes(pcm.tobytes())
    print(f"{path}: {len(left) / RATE:.1f}s")


if __name__ == "__main__":
    destination = Path(__file__).resolve().parents[1] / "assets/music/demo"
    render(destination / "pallet.wav", PALLET,
           ((57, 61, 64), (59, 62, 66), (54, 57, 61), (52, 56, 59),
            (57, 61, 64), (50, 54, 57), (59, 62, 66), (52, 56, 59)), 98)
    render(destination / "route1.wav", ROUTE,
           ((55, 59, 62), (60, 64, 67), (59, 62, 66), (55, 59, 62),
            (52, 55, 59), (60, 64, 67), (59, 62, 66), (50, 54, 57)), 126)
