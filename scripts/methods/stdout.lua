function to10bit(x)
	return math.floor(4095 * x)
end

function write(leds)
	for k,v in pairs(leds) do
		pinfo("LED" .. tostring(k - 1) .. " r=" .. tostring(to10bit(v.r)) .. " g=" .. tostring(to10bit(v.g)) .. " b=" .. tostring(to10bit(v.b)))
	end
end

pdebug("host version: " .. hyperion_params.host.version)
