-- Adapted from nbody-lua-1 under ../LICENSE.txt.

local DAYS_PER_YEAR = 365.24
local DELTA_TIME = 0.01
local SOLAR_MASS = 4 * math.pi * math.pi
local squareRoot = math.sqrt

local function createBodies()
	return {
		{
			positionX = 0,
			positionY = 0,
			positionZ = 0,
			velocityX = 0,
			velocityY = 0,
			velocityZ = 0,
			mass = SOLAR_MASS,
		},
		{
			positionX = 4.841431442464721,
			positionY = -1.1603200440274284,
			positionZ = -0.10362204447112311,
			velocityX = 0.001660076642744037 * DAYS_PER_YEAR,
			velocityY = 0.007699011184197404 * DAYS_PER_YEAR,
			velocityZ = -0.0000690460016972063 * DAYS_PER_YEAR,
			mass = 0.0009547919384243266 * SOLAR_MASS,
		},
		{
			positionX = 8.34336671824458,
			positionY = 4.124798564124305,
			positionZ = -0.4035234171143214,
			velocityX = -0.002767425107268624 * DAYS_PER_YEAR,
			velocityY = 0.004998528012349172 * DAYS_PER_YEAR,
			velocityZ = 0.000023041729757376393 * DAYS_PER_YEAR,
			mass = 0.0002858859806661308 * SOLAR_MASS,
		},
		{
			positionX = 12.894369562139131,
			positionY = -15.111151401698631,
			positionZ = -0.22330757889265573,
			velocityX = 0.002964601375647616 * DAYS_PER_YEAR,
			velocityY = 0.0023784717395948095 * DAYS_PER_YEAR,
			velocityZ = -0.000029658956854023756 * DAYS_PER_YEAR,
			mass = 0.00004366244043351563 * SOLAR_MASS,
		},
		{
			positionX = 15.379697114850911,
			positionY = -25.919314609987964,
			positionZ = 0.17925877295037118,
			velocityX = 0.0026806777249038932 * DAYS_PER_YEAR,
			velocityY = 0.001628241700382423 * DAYS_PER_YEAR,
			velocityZ = -0.00009515922545197159 * DAYS_PER_YEAR,
			mass = 0.000051513890204661145 * SOLAR_MASS,
		},
	}
end

local function offsetMomentum(bodies)
	local momentumX = 0
	local momentumY = 0
	local momentumZ = 0

	for bodyIndex = 1, #bodies do
		local body = bodies[bodyIndex]
		local bodyMass = body.mass
		momentumX = momentumX + body.velocityX * bodyMass
		momentumY = momentumY + body.velocityY * bodyMass
		momentumZ = momentumZ + body.velocityZ * bodyMass
	end

	bodies[1].velocityX = -momentumX / SOLAR_MASS
	bodies[1].velocityY = -momentumY / SOLAR_MASS
	bodies[1].velocityZ = -momentumZ / SOLAR_MASS
end

local function advance(bodies, bodyCount, deltaTime)
	for bodyIndex = 1, bodyCount do
		local body = bodies[bodyIndex]
		local positionX = body.positionX
		local positionY = body.positionY
		local positionZ = body.positionZ
		local bodyMass = body.mass
		local velocityX = body.velocityX
		local velocityY = body.velocityY
		local velocityZ = body.velocityZ

		for otherBodyIndex = bodyIndex + 1, bodyCount do
			local otherBody = bodies[otherBodyIndex]
			local distanceX = positionX - otherBody.positionX
			local distanceY = positionY - otherBody.positionY
			local distanceZ = positionZ - otherBody.positionZ
			local distance = squareRoot(
				distanceX * distanceX + distanceY * distanceY + distanceZ * distanceZ
			)
			local magnitude = deltaTime / (distance * distance * distance)

			local otherMagnitude = otherBody.mass * magnitude
			velocityX = velocityX - distanceX * otherMagnitude
			velocityY = velocityY - distanceY * otherMagnitude
			velocityZ = velocityZ - distanceZ * otherMagnitude

			local bodyMagnitude = bodyMass * magnitude
			otherBody.velocityX = otherBody.velocityX + distanceX * bodyMagnitude
			otherBody.velocityY = otherBody.velocityY + distanceY * bodyMagnitude
			otherBody.velocityZ = otherBody.velocityZ + distanceZ * bodyMagnitude
		end

		body.velocityX = velocityX
		body.velocityY = velocityY
		body.velocityZ = velocityZ
	end

	for bodyIndex = 1, bodyCount do
		local body = bodies[bodyIndex]
		body.positionX = body.positionX + deltaTime * body.velocityX
		body.positionY = body.positionY + deltaTime * body.velocityY
		body.positionZ = body.positionZ + deltaTime * body.velocityZ
	end
end

local function energy(bodies)
	local totalEnergy = 0

	for bodyIndex = 1, #bodies do
		local body = bodies[bodyIndex]
		local velocityX = body.velocityX
		local velocityY = body.velocityY
		local velocityZ = body.velocityZ
		local bodyMass = body.mass
		totalEnergy = totalEnergy
			+ 0.5 * bodyMass * (velocityX * velocityX + velocityY * velocityY + velocityZ * velocityZ)

		for otherBodyIndex = bodyIndex + 1, #bodies do
			local otherBody = bodies[otherBodyIndex]
			local distanceX = body.positionX - otherBody.positionX
			local distanceY = body.positionY - otherBody.positionY
			local distanceZ = body.positionZ - otherBody.positionZ
			local distance = squareRoot(
				distanceX * distanceX + distanceY * distanceY + distanceZ * distanceZ
			)
			totalEnergy = totalEnergy - bodyMass * otherBody.mass / distance
		end
	end

	return totalEnergy
end

local function benchmark(stepCount)
	local bodies = createBodies()
	local bodyCount = #bodies
	offsetMomentum(bodies)
	energy(bodies)
	for _ = 1, stepCount do
		advance(bodies, bodyCount, DELTA_TIME)
	end
	return energy(bodies)
end
