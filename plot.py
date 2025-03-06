import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Line3DCollection

def read_off(filename):
    with open(filename) as file:
        if 'OFF' != file.readline().strip():
            raise('Not a valid OFF header')
        n_verts, n_faces, n_dontknow = tuple([int(s) for s in file.readline().strip().split(' ')])
        verts = [[float(s) for s in file.readline().strip().split(' ')] for i_vert in range(n_verts)]
        faces = [[int(s) for s in file.readline().strip().split(' ')][1:] for i_face in range(n_faces)]
        return verts, faces

def read_chain(filename):
    chain = []

    with open(filename) as file:
        for line in file:
            parts = line.split() 
            chain.append([int(x) for x in parts])

    return chain

# Load mesh and chains
V, F = read_off("Plane.off")
c1 = read_chain("Chain1.txt")
c2 = read_chain("Chain2.txt")
c3 = read_chain("Chain3.txt")
c_median = read_chain("median.txt")

# Setup figure
fig = plt.figure(figsize=(16,9))
ax = fig.add_subplot(projection='3d')

# Plot the mesh
X = [v[0] for v in V]
Y = [v[2] for v in V]
Z = [v[1] for v in V]
ax.plot_trisurf(X, Y, Z, triangles=F, color=(0,0,0,0), edgecolor='gray')
#ax.scatter3D(X, Y, Z, color='black')

# Plot the chains
for color, chain in [("green", c1), ("blue", c2), ("red", c3), ("black", c_median)]:
    if len(chain) == 0:
        continue
    data = [[[V[i][0], V[i][2], V[i][1]] for i in x] for x in chain]
    lc = Line3DCollection(data, color=color)
    lc.set_linewidths(5 if color == "black" else 2.5)
    ax.add_collection3d(lc)

ax.view_init(elev=90, azim=0)
ax.grid(False)
ax.axis('off')
ax.set_xticks([])
ax.set_yticks([])
ax.set_zticks([])
plt.show()
