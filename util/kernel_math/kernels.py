"""Calculate kernel gradients and generate test case"""
from sympy import *
r, h, x, y = symbols('r h x y')
q = r / h

def simp(expr):
    global x, y, r, h
    Q = symbols('Q')
    ret = expr.subs(sqrt(x**2+y**2), r)
    ret = ret.subs(x**2, r**2-y**2)
    ret = ret.subs(r, Q*h)
    return simplify(ret)

alpha = Rational(7, 4) / pi * (1/h)**2
kernel2 = (1 - Rational(1, 2) * q)**4 * (2 * q + 1) * alpha
xyexpr = kernel2.subs([(r, sqrt(x**2+y**2))])
grad_x = diff(xyexpr, x)
grad_y = diff(xyexpr, y)
print("Grad x: ", simp(grad_x / alpha))
print("Grad y: ", simp(grad_y / alpha))

eval_point = {
    x: 1,
    y: 2,
    r: sqrt(5),
    h: 10
}
grad_x_1 = grad_x.evalf(subs=eval_point)
grad_y_1 = grad_y.evalf(subs=eval_point)
print(eval_point)
print("grad_x = {}".format(grad_x_1))
print("grad_y = {}".format(grad_y_1))

Q = symbols('q')
laplacian = diff(grad_x, x) + diff(grad_y, y)
print(simp(laplacian/alpha))
laplacian_1 = laplacian.evalf(subs=eval_point)
print("laplacian_1 = {}".format(laplacian_1))