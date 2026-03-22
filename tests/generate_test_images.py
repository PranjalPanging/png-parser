import os
import numpy as np
from PIL import Image

os.makedirs("tests/images", exist_ok=True)

img = Image.fromarray(
    np.random.randint(0, 255, (1000, 1000, 3), dtype=np.uint8)
)
img.save("tests/images/carrier_large.png")
print("✓ carrier_large.png")

img = Image.new("RGB", (50, 50), color=(100, 149, 237))
img.save("tests/images/carrier_small.png")
print("✓ carrier_small.png")

img = Image.fromarray(
    np.random.randint(0, 255, (500, 500, 3), dtype=np.uint8)
)
img.save("tests/images/carrier_bmp.bmp")
print("✓ carrier_bmp.bmp")

img.save("tests/images/carrier_tiff.tiff")
print("✓ carrier_tiff.tiff")

with open("tests/images/not_image.txt", "w") as f:
    f.write("this is not an image")
print("✓ not_image.txt")

print("\nAll test images generated.")