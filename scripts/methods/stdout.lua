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
		pinfo("LED" .. tostring(k - 1) .. " r=" .. tostring(tobits(v.r)) .. " g=" .. tostring(tobits(v.g)) .. " b=" .. tostring(tobits(v.b)))
	end
end
