import numpy as np

RADIUS = 1
SEGMENTS = 128
RINGS = 64
CHAINS_THETA = [ np.deg2rad((360 / SEGMENTS) * j) for j in [ np.floor(SEGMENTS/2) - 10, np.floor(SEGMENTS/2) - 6, np.floor(SEGMENTS/2) + 10] ]

def spherical_coordinates(r, theta, phi):
    x = r * np.sin(phi) * np.cos(theta);
    y = r * np.sin(phi) * np.sin(theta);
    z = r * np.cos(phi);
    return [x,y,z]

# Generate vertices of sphere
V = []
for j in range(1, RINGS):
    phi = np.deg2rad((180 / RINGS) * j)
    for i in range(SEGMENTS):
        theta = np.deg2rad((360 / SEGMENTS) * i)
        V.append(spherical_coordinates(RADIUS, theta, phi))

# Don't forget north/south poles
V.append([0,0,RADIUS]);
V.append([0,0,-RADIUS]);

# Generate triangles
T = []
for j in range(RINGS):
    for i in range(SEGMENTS):
        if j == RINGS-2: # North cap
            T.append([len(V)-2, i, (i+1) % SEGMENTS])
        elif j == RINGS-1: # South cap
            T.append([len(V)-1, (RINGS-2)*SEGMENTS + i, (RINGS-2)*SEGMENTS + (i+1) % SEGMENTS])
        else:
            T.append([j*SEGMENTS + i, j*SEGMENTS + ((i+1) % SEGMENTS), (j+1)*SEGMENTS + i])
            T.append([j*SEGMENTS + ((i+1) % SEGMENTS), (j+1)*SEGMENTS + ((i+1) % SEGMENTS), (j+1)*SEGMENTS + i])

# Save mesh
with open("Sphere.off", "w") as fp:
    fp.write("OFF\n");
    fp.write("{} {} 0".format(len(V), len(T)));
    for v in V:
        fp.write("\n{} {} {}".format(v[0], v[1], v[2]))
    for t in T:
        fp.write("\n3 {} {} {}".format(t[0], t[1], t[2]))

# Generate currents
for i, theta in enumerate(CHAINS_THETA):
    with open("SphereCurrent{}.txt".format(i+1), "w") as fp:
        fp.write("0.0 0.0 {}".format(RADIUS))
        for i in range(1, RINGS):
            phi = np.deg2rad((180 / RINGS) * i)
            v = spherical_coordinates(RADIUS, theta, phi)
            fp.write("\n{} {} {}".format(v[0], v[1], v[2]))
        fp.write("\n0.0 0.0 {}".format(-RADIUS))
