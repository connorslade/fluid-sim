using CairoMakie
using Makie

include("common.jl")

PRESSURE_THRESHOLD = 0.25
AREA_THRESHOLD = 200

function locate_blobs(state, pressure_clip, area_clip)
	pressures = imfilter(map(cell -> cell.pressure, state), Kernel.gaussian(6))
	in_bounds = (x, y) -> x > 0 && x <= size(state, 1) && y > 0 && y <= size(state, 2)

	seen = Set()
	queue = []
	blobs = []

	for x in 1:size(state, 1), y in 1:size(state, 2)
		threshold = (x, y) -> pressures[x, y] > pressure_clip

		if !in((x, y), seen) && threshold(x, y)
			push!(queue, (x, y))
			blob = []

			while !isempty(queue)
				(cx, cy) = pop!(queue)
				if !in((cx, cy), seen) && in_bounds(cx, cy) && threshold(cx, cy)
					push!(blob, (cx, cy))
					push!(seen, (cx, cy))

					for delta in [(-1, 0), (1, 0), (0, -1), (0, 1)]
						dx, dy = delta
						nx, ny = cx + dx, cy + dy
						if in_bounds(nx, ny) && threshold(nx, ny) && !in((nx, ny), seen)
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

	blobs = [(mean(x for (x, _) in blob), mean(y for (_, y) in blob)) for blob in blobs if length(blob) >= area_clip]
	return pressures, blobs
end

states = readdir("states")
states = filter(state -> endswith(state, ".bin"), states)
states = sort(states, by = x -> parse(Int, splitext(basename(x))[1]))

n = ceil(Int, sqrt(length(states)))
f = Figure(size = (800, 800))

data = []
for (i, state_file) in enumerate(states)
	state = load_state(state_file)
	pressure, blobs = locate_blobs(state, PRESSURE_THRESHOLD, AREA_THRESHOLD)
	center_x, center_y = size(state, 1) / 2, size(state, 2) / 2
	angles = map(blob -> atan(blob[2] - center_y, blob[1] - center_x), blobs)
	append!(data, [(i-1, angle) for angle in angles])

	fig = f[div(i-1, n), mod(i-1, n)]
	ax = Axis(fig, aspect=DataAspect(), title="θ=$(map(θ -> round(θ, digits=2), angles))")
	heatmap!(ax, map(x -> x > PRESSURE_THRESHOLD, pressure))
	hidedecorations!(ax)

	for (x, y) in blobs
		scatter!(ax, [x], [y], color=:red)
	end
end

for (h, angle) in data
	println("$(2*h),$angle")
end

display(f)