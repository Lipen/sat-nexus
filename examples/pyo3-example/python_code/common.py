from pyeda.boolalg.minimization import espresso_exprs
from pyeda.inter import exprvar, And, Or


def bool2int(b):
    return 1 if b else 0


def bool2sign(b):
    return -1 if b else 1


def signed(x, s):
    return bool2sign(s) * x


def backdoor_to_clauses(easy):
    # Note: here, 'dnf' represents the negation of characteristic function,
    #       because we use "easy" tasks here.
    dnf = cubes_to_dnf(easy)
    (min_dnf,) = minimize_dnf(dnf)
    min_cnf = (~min_dnf).to_cnf()  # here, we negate the function back
    clauses = cnf_to_clauses(min_cnf)
    return clauses


def cubes_to_dnf(cubes):
    variables = [abs(x) for x in cubes[0]]
    for cube in cubes:
        assert variables == [abs(lit) for lit in cube], f"cube={cube}, vars={variables}"

    # print(f"Converting {len(cubes)} cubes over {len(variables)} variables into DNF...")
    var_map = dict()
    cubes_expr = []

    for cube in cubes:
        lits_expr = []
        for lit in cube:
            var = abs(lit)
            if var not in var_map:
                var_map[var] = exprvar("x", var)
            if lit < 0:
                lits_expr.append(~var_map[var])
            else:
                lits_expr.append(var_map[var])
        cubes_expr.append(And(*lits_expr))

    dnf = Or(*cubes_expr)
    assert dnf.is_dnf()
    return dnf


def minimize_dnf(dnf):
    # print(f"Minimizing DNF via Espresso...")
    min_dnf = espresso_exprs(dnf)
    return min_dnf


def cnf_to_clauses(cnf):
    # print("Converting CNF into clauses...")

    assert cnf.is_cnf()

    litmap, nvars, clauses = cnf.encode_cnf()
    result = []
    for clause in clauses:
        c = []
        for lit in clause:
            v = litmap[abs(lit)].indices[0]  # 1-based variable index
            s = lit < 0  # sign
            c.append(signed(v, s))
        c.sort(key=lambda x: abs(x))
        result.append(c)

    clauses = result
    clauses.sort(key=lambda x: (len(x), tuple(map(abs, x))))
    # print(
    #     f"Total {len(clauses)} clauses: {sum(1 for clause in clauses if len(clause) == 1)} units, {sum(1 for clause in clauses if len(clause) == 2)} binary, {sum(1 for clause in clauses if len(clause) == 3)} ternary, {sum(1 for clause in clauses if len(clause) > 3)} larger"
    # )
    return clauses
