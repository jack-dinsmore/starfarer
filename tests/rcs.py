import numpy as np
from scipy import optimize

NUM_THRUSTERS = 4
MAX_FORCE = 1
COS_GAMMA = 0.1
SLOPE = 1e10

# Generate positions and normals
positions = np.random.randn(NUM_THRUSTERS * 3).reshape(-1, 3)
normals = np.random.randn(NUM_THRUSTERS * 3).reshape(-1, 3)
for i in range(NUM_THRUSTERS):
    normals[i] /= np.linalg.norm(normals[i])

m = np.zeros((6, 3 * NUM_THRUSTERS))
# Force part
for i in range(3):
    for t in range(NUM_THRUSTERS):
        for j in range(3):
            m[i, 3 * t + j] = (i == j)

# Torque part
for t, pos in enumerate(positions):
    m[3, t * 3] = 0
    m[3, t * 3 + 1] = -pos[2]
    m[3, t * 3 + 2] = pos[1]

    m[3 + 1, t * 3] = pos[2]
    m[3 + 1, t * 3 + 1] = 0
    m[3 + 1, t * 3 + 2] = -pos[0]

    m[3 + 2, t * 3] = -pos[1]
    m[3 + 2, t * 3 + 1] = pos[0]
    m[3 + 2, t * 3 + 2] = 0
    
mp = np.linalg.pinv(m)

a = np.identity(3 * NUM_THRUSTERS) - mp @ m

def get_forces(target):
    def forces_free(w):
        return mp @ target + a @ w
    def discard(f):
        add = 0
        for t in range(NUM_THRUSTERS):
            force = f[t*3:(t+1)*3]
            mag = np.linalg.norm(force)
            angle = np.dot(force, normals[t]) / mag
            if mag > MAX_FORCE:
                add += (mag - MAX_FORCE) * SLOPE
            if angle < COS_GAMMA:
                add += (COS_GAMMA - angle) * SLOPE
        return add

    def minimize(w):
        f = forces_free(w)
        fuel = discard(f)
        for t in range(NUM_THRUSTERS):
            force = f[t*3:(t+1)*3]
            fuel += np.linalg.norm(force)
        return fuel
    
    result = optimize.minimize(minimize, np.zeros(3 * NUM_THRUSTERS))
    forces = forces_free(result.x)
    return forces, discard(forces) < 0.01

def get_maximal_radius(normal):
    renorm = 1 / np.linalg.norm(normal)
    forces, success = get_forces(normal * renorm)
    while not success:
        renorm /= 2
        forces, success = get_forces(normal * renorm)
    max_force = 0
    for t in range(NUM_THRUSTERS):
        force = forces[t*3:(t+1)*3]
        max_force = max(max_force, np.linalg.norm(force))

    radius = renorm * NUM_THRUSTERS / max_force
    print(radius)
    return radius


phi = (1 + np.sqrt(5)) / 2
POINTS = [
    (0, 1, phi),
    (0, 1, -phi),
    (0, -1, phi),
    (0, -1, -phi),

    (1, phi, 0),
    (1, -phi, 0),
    (-1, phi, 0),
    (-1, -phi, 0),

    (phi, 0, 1),
    (-phi, 0, 1),
    (phi, 0, -1),
    (-phi, 0, -1),
]
indices = [
    (0, 2, 8),
    (0, 2, 9),
    (1, 3, 10),
    (1, 3, 11),

    (4, 6, 0),
    (4, 6, 1),
    (5, 7, 2),
    (5, 7, 3),

    (8, 10, 4),
    (8, 10, 5),
    (9, 11, 6),
    (9, 11, 7),
    
]
scatter_x = [[], []]
scatter_y = [[], []]
scatter_z = [[], []]
zeros = np.zeros(3)
max_force_radius = 0
max_torque_radius = 0

for p in POINTS:
    pforce = np.array(p) * get_maximal_radius(np.append(p, zeros)) / np.linalg.norm(p)
    ptorque = np.array(p) * get_maximal_radius(np.append(zeros, p)) / np.linalg.norm(p)
    max_force_radius = max(max_force_radius, np.linalg.norm(pforce))
    max_torque_radius = max(max_force_radius, np.linalg.norm(ptorque))
    scatter_x[0].append(pforce[0])
    scatter_y[0].append(pforce[1])
    scatter_z[0].append(pforce[2])
    scatter_x[1].append(ptorque[0])
    scatter_y[1].append(ptorque[1])
    scatter_z[1].append(ptorque[2])

import matplotlib.pyplot as plt

def plot_triangles(x, y, color, marker, max_r):
    for i in indices:
        plt.plot([x[i[0]], x[i[1]], x[i[2]], x[i[0]]], [y[i[0]], y[i[1]], y[i[2]], y[i[0]]],
            marker=marker, color=color)
    plt.xlim(-max_r, max_r)
    plt.ylim(-max_r, max_r)

plt.figure(figsize=(5,5))
plot_triangles(scatter_x[0], scatter_y[0], "C0", 'o', max_force_radius)
plt.xlabel("x")
plt.ylabel("y")
plt.title("Force")

plt.figure(figsize=(5,5))
plot_triangles(scatter_x[0], scatter_z[0], "C0", 'o', max_force_radius)
plt.xlabel('x')
plt.ylabel('z')
plt.title("Force")

plt.figure(figsize=(5,5))
plot_triangles(scatter_x[1], scatter_y[1], "C1", 'x', max_torque_radius)
plt.xlabel("x")
plt.ylabel("y")
plt.title("Torque")

plt.figure(figsize=(5,5))
plot_triangles(scatter_x[1], scatter_z[1], "C1", 'x', max_torque_radius)
plt.xlabel('x')
plt.ylabel('z')
plt.title("Torque")

plt.show()