WAVEFRONT = "sphere.obj"
MESH = "Sphere"
CURRENTS = ["Current1", "Current2"]

def read_obj(filename):
    with open(filename) as file:
        objects = {}
        current_obj = None
        idx_offset = 1
        for line in file.readlines():
            parts = line.split()
            if parts[0] == 'o':
                if current_obj != None:
                    idx_offset += len(objects[current_obj]["v"])
                current_obj = parts[1]
                objects[current_obj] = { "v": [], "l": [], "f": [] }
            elif parts[0] == 'v':
                objects[current_obj]["v"].append([float(parts[1]), float(parts[2]), float(parts[3])])
            elif parts[0] == 'l':
                objects[current_obj]["l"].append([int(parts[1]) - idx_offset, int(parts[2]) - idx_offset])
            elif parts[0] == 'f':
                objects[current_obj]["f"].append([int(x.split("/")[0]) - idx_offset for x in parts[1:]])

        return objects

def distance(a, b):
    return (a[0] - b[0])**2 + (a[1]-b[1])**2 + (a[2]-b[2])**2

def closest_point(target, points):
    closest = None
    min_dist = float('inf')

    for i, pt in enumerate(points):
        d = distance(target, pt)
        if d < min_dist:
            min_dist = d
            closest = pt 

    return closest

objects = read_obj(WAVEFRONT)

# Create the OFF file
with open("{}.off".format(MESH), "w") as f:
    f.write("OFF\n")
    f.write("{} {} 0\n".format(len(objects[MESH]["v"]), len(objects[MESH]["f"])))
    for v in objects[MESH]["v"]:
        f.write("{} {} {}\n".format(v[0], v[1], v[2]))
    for face in objects[MESH]["f"]:
        f.write("3 {} {} {}\n".format(face[0], face[1], face[2]))

# Create the chains
for current in CURRENTS:
    vertices = []
    for i, edge in enumerate(objects[current]["l"]):
        a = closest_point(objects[current]["v"][edge[0]], objects[MESH]["v"])
        b = closest_point(objects[current]["v"][edge[1]], objects[MESH]["v"])
        if a not in vertices:
            vertices.append(a);
        if b not in vertices:
            vertices.append(b);

    sorted_vertices = sorted(vertices, key=lambda x: x[1])
    print(sorted_vertices)

    with open("Sphere{}.txt".format(current), "w") as f:
        for i, v in enumerate(sorted_vertices):
            if i > 0:
                f.write("\n")
            f.write("{} {} {}".format(v[0], v[1], v[2]))

