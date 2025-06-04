state = load_state("1748997635.bin")
center_x, center_y = size(state, 1) / 2, size(state, 2) / 2

blobs = locate_blobs(state)
println("[*] Found $(length(blobs)) blobs")
angles = map(blob -> atan(blob[2] - center_y, blob[1] - center_x), blobs)

f, ax, hm = heatmap(map(x -> x.pressure, state))
hidedecorations!(ax)

for (x, y) in blobs
	scatter!(ax, [x], [y], color=:red, markersize=10)
end

f