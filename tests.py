from pathlib import Path
import pytest
import subprocess


@pytest.fixture(scope="session", autouse=True)
def check_bin_dir():
    if not (Path("tests/bin").exists()):
        Path("tests/bin").mkdir()


def run(binary, toy, c, expected):
    binary = "tests/bin/" + binary
    toy = "tests/" + toy
    c = "tests/" + c

    # Dump IR first
    result = subprocess.run(
        ["cargo", "run", "--", "--dump-ir", toy], capture_output=True, text=True
    )
    assert result.returncode == 0

    # Compile the program
    result = subprocess.run(["cargo", "run", "--", toy], capture_output=True, text=True)
    assert result.returncode == 0

    # Link the assembly with the C code
    result = subprocess.run(["clang", "-o", binary, c, toy + ".s"], capture_output=True)
    assert result.returncode == 0

    # Run the executable
    result = subprocess.run([binary], capture_output=True, text=True)
    out = result.stdout

    # Compare the output
    assert out == expected


fib_ans = "1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n"


def test_fib1():
    run("fib1", "fib1.toy", "fib.c", fib_ans)


def test_fib2():
    run("fib2", "fib2.toy", "fib.c", fib_ans)


factorial_ans = "1\n1\n2\n6\n24\n120\n720\n5040\n40320\n362880\n"


def test_factorial1():
    run("factorial1", "factorial1.toy", "factorial.c", factorial_ans)


def test_factorial2():
    run("factorial2", "factorial2.toy", "factorial.c", factorial_ans)


def test_prime():
    run(
        "prime",
        "prime.toy",
        "prime.c",
        "2\n3\n5\n7\n11\n13\n17\n19\n23\n29\n31\n37\n41\n43\n47\n53\n59\n61\n67\n71\n73\n79\n83\n89\n97\n",
    )


def test_sum():
    run("sum", "sum.toy", "sum.c", "0\n1\n3\n6\n10\n15\n21\n28\n36\n45\n")


gcd_ans = "0\n1\n1\n5\n6\n12\n28\n42\n"


def test_gcd1():
    run("gcd", "gcd1.toy", "gcd.c", gcd_ans)


def test_gcd2():
    run("gcd", "gcd2.toy", "gcd.c", gcd_ans)
