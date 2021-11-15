from binaryninja import *
import re, subprocess

def load_dbg_file(bv, function):
    rex = re.compile("^([FGS]) ([0-9a-f]{8}) (.*)$")

    # Prompt for debug file input
    dbg_file = interaction \
        .get_open_filename_input("Debug file",
        "COFF Debug Files (*.dbg *.db_)")

    if dbg_file:
        # Parse the debug file
        output = subprocess.check_output(["dbgparse", dbg_file]).decode()
        for line in output.splitlines():
            (typ, addr, name) = rex.match(line).groups()
            addr = bv.start + int(addr, 16)

            (mangle_typ, mangle_name) = demangle.demangle_ms(bv.arch, name)
            if type(mangle_name) == list:
                mangle_name = "::".join(mangle_name)

            if typ == "F":
                # Function
                bv.create_user_function(addr)
                bv.define_user_symbol(Symbol(SymbolType.FunctionSymbol, addr, mangle_name, raw_name=name))
                if mangle_typ != None:
                    bv.get_function_at(addr).function_type = mangle_typ
            elif typ == "G":
                # Global
                bv.define_user_symbol(Symbol(SymbolType.DataSymbol, addr, mangle_name, raw_name=name))
            elif typ == "S":
                # Sourceline
                bv.set_comment_at(addr, name)

PluginCommand.register_for_address("Load COFF DBG file", "Load COFF .DBG file from disk", load_dbg_file)

