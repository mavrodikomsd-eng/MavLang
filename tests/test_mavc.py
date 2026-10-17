def test_transpile_print():
    import importlib.util
    spec = importlib.util.spec_from_file_location("mavc", r"C:\1\аап\mavc.py")
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    rs = m.parse_and_transpile('fn main() {\n print("hi");\n}\n', "borrow")
    assert "println!" in rs
    assert "fn main" in rs
