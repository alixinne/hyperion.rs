import time

# Get the parameters
fadeTime   = float(hyperion.args.get('fade-time', 5.0))
colorStart = hyperion.args.get('color-start', (255,174,11))
colorEnd   = hyperion.args.get('color-end', (100,100,100))

color_step = (
	(colorEnd[0] - colorStart[0]) / 256.0,
	(colorEnd[1] - colorStart[1]) / 256.0,
	(colorEnd[2] - colorStart[2]) / 256.0
)

# fade color
for step in range(256):
	if hyperion.abort():
		break

	hyperion.setColor(
		min(max(int(colorStart[0] + color_step[0] * step), 0), 255),
		min(max(int(colorStart[1] + color_step[1] * step), 0), 255),
		min(max(int(colorStart[2] + color_step[2] * step), 0), 255),
	)
	time.sleep( fadeTime / 256 )

# maintain color until effect end
hyperion.setColor(colorEnd[0], colorEnd[1], colorEnd[2])
while not hyperion.abort():
	time.sleep(1)

