using DataStructures
using ImageFiltering
using Statistics

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
