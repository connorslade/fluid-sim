using CairoMakie
using DataStructures
using Makie
using Statistics

THRESHOLD = 0.2

struct Cell
	pressure::Float32
	velocity::NTuple{2, Float32}
end

function load_state(name)
	state = read("states/" * name)
	(width, height) = reinterpret(UInt32, state[1:8])
	println("[*] Loading state `$name` ($width × $height)")
	return reshape(reinterpret(Cell, state[9:end]), (width, height))
end

function locate_blobs(state)
	seen = Set()
	queue = []
	blobs = []

	for x in 1:size(state, 1)
		for y in 1:size(state, 2)
			if !((x, y) in seen) && state[x, y].pressure > THRESHOLD
				push!(queue, (x, y))
				blob = []

				while !isempty(queue)
					(cx, cy) = pop!(queue)
					if !in((cx, cy), seen) && cx > 0 && cx <= size(state, 1) && cy > 0 && cy <= size(state, 2) && state[cx, cy].pressure > THRESHOLD
						push!(blob, (cx, cy))
						push!(seen, (cx, cy))

						for delta in [(-1, 0), (1, 0), (0, -1), (0, 1)] 
							dx, dy = delta
							nx, ny = cx + dx, cy + dy
							if nx > 0 && nx <= size(state, 1) && ny > 0 && ny <= size(state, 2) && state[nx, ny].pressure > THRESHOLD && !in((nx, ny), seen)
								push!(queue, (nx, ny))
							end
						end
					end
				end

				if !isempty(blob)
					push!(blobs, blob)
				end
			end
		end
	end

	return map(blob -> (mean(map(coord -> coord[1], blob)), mean(map(coord -> coord[2], blob))), blobs)
end
