pdebug("host version: " .. hyperion_params.host.version)
pdebug("lua stdout method initializing")

if hyperion_params["bits"] ~= nil then
	bits = hyperion_params.bits
else
	bits = 8
end

function tobits(x)
	return math.floor(((1 << bits) - 1) * x)
end

function write(leds)
	for k,v in pairs(leds) do
		local msg = ""
		for i,q in pairs(v) do
			msg = msg .. " " .. tostring(i - 1) .. "=" .. tostring(tobits(q))
		end

		pinfo("LED" .. tostring(k - 1) .. msg)
	end
end
