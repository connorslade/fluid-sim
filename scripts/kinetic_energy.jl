using Makie
using CairoMakie

include("common.jl")

states = filter(state -> endswith(state, ".bin"), readdir("states"))
states = sort(states, by = x -> parse(Int, splitext(basename(x))[1]))

energies = []

for path in states
    state = load_state(path)
    energy = reduce(+, map(x -> sqrt(x.velocity[1]^2 + x.velocity[2]^2), state))
    
    push!(energies, energy)
end

f = Figure()
ax = Axis(f[1, 1], title = "Velocity over Time", xlabel = "Ticks", ylabel = "Velocity")
lines!(ax, 1:length(energies), energies)

f