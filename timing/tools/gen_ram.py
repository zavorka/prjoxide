import random

print("module top(input [3:0] clk, ce, rst, input [7:0] d, output [7:0] q);")
data = ["d[{}]".format(i) for i in range(8)]

def data_bits(N):
    return "{{{}}}".format(", ".join([random.choice(data) for i in range(N)]))
def clock_port(name):
    print("        .{}(clk[{}]),".format(name, random.randint(0, 3)))
def ce_port(name):
    print("        .{}(ce[{}]),".format(name, random.randint(0, 3)))
def rst_port(name):
    print("        .{}(rst[{}]),".format(name, random.randint(0, 3)))
def data_port(name, N):
    print("        .{}({}),".format(name, data_bits(N)))
def output_port(name, i, j, N, last=False):
    print("        .{}(d_{}[{} +: {}]){}".format(name, i, j, N, "" if last else ","))

def get_next_data(i, N):
    print("    wire [{}:0] d_{};".format(N-1, i))
    return ["d_{}[{}]".format(i, j) for j in range(N)]

N = 80
M = 2

for i in range(N):
    prim = random.choice(["DP16K", "PDP16K", "PDPSC16K", "SP16K", "FIFO16K"])
    if prim == "DP16K":
        next_data = get_next_data(i, 18+18)
        print("    DP16K ram_{} (".format(i))
        clock_port("CLKA")
        clock_port("CLKB")
        ce_port("CEA")
        ce_port("CEB")
        ce_port("WEA")
        ce_port("WEB")
        rst_port("RSTA")
        rst_port("RSTB")
        data_port("CSA", 3)
        data_port("CSB", 3)
        data_port("ADA", 14)
        data_port("ADB", 14)
        data_port("DIA", 18)
        data_port("DIB", 18)
        output_port("DOA", i, 0,  18, False)
        output_port("DOB", i, 18, 18, True)
        print("    );")
    elif prim == "PDP16K":
        next_data = get_next_data(i, 36+2)
        print("    PDP16K ram_{} (".format(i))
        clock_port("CLKW")
        clock_port("CLKR")
        ce_port("CEW")
        ce_port("CER")
        rst_port("RST")
        data_port("CSW", 3)
        data_port("CSR", 3)
        data_port("ADW", 14)
        data_port("ADR", 14)
        data_port("DI", 36)
        output_port("DO", i, 0,  36, False)
        output_port("ONEBITERR", i, 36,  1, False)
        output_port("TWOBITERR", i, 37,  1, True)
        print("    );")
    elif prim == "PDPSC16K":
        next_data = get_next_data(i, 36+2)
        print("    PDPSC16K ram_{} (".format(i))
        clock_port("CLK")
        ce_port("CEW")
        ce_port("CER")
        rst_port("RST")
        data_port("CSW", 3)
        data_port("CSR", 3)
        data_port("ADW", 14)
        data_port("ADR", 14)
        data_port("DI", 36)
        output_port("DO", i, 0,  36, False)
        output_port("ONEBITERR", i, 36,  1, False)
        output_port("TWOBITERR", i, 37,  1, True)
        print("    );")
    elif prim == "SP16K":
        next_data = get_next_data(i, 18)
        print("    SP16K ram_{} (".format(i))
        clock_port("CLK")
        ce_port("CE")
        ce_port("WE")
        rst_port("RST")
        data_port("CS", 3)
        data_port("AD", 14)
        data_port("DI", 18)
        output_port("DO", i, 0,  18, True)
        print("    );")
    elif prim == "FIFO16K":
        next_data = get_next_data(i, 18+18+6)
        print("    FIFO16K ram_{} (".format(i))
        clock_port("CKA")
        clock_port("CKB")
        ce_port("CEA")
        ce_port("CEB")
        # ce_port("WEA")
        # ce_port("WEB")
        rst_port("RSTA")
        rst_port("RSTB")
        data_port("CSA", 3)
        data_port("CSB", 3)
        data_port("DIA", 18)
        data_port("DIB", 18)
        output_port("DOA", i, 0,  18, False)
        output_port("DOB", i, 18, 18, False)
        output_port("ONEBITERR", i, 36,  1, False)
        output_port("TWOBITERR", i, 37,  1, False)
        output_port("ALMOSTFULL", i, 38,  1, False)
        output_port("FULL", i, 39,  1, False)
        output_port("ALMOSTEMPTY", i, 40,  1, False)
        output_port("EMPTY", i, 41,  1, True)
        print("    );")
    else:
        assert False
    data = next_data

for i in range(N, N+M):
    lram_prim = random.choice(["DPSC512K", "PDPSC512K", "SP512K"])
    if lram_prim == "DPSC512K":
        next_data = get_next_data(i, 32+32+4)
        print("    DPSC512K lram_{} (".format(i))
        clock_port("CLK")
        ce_port("CEA")
        ce_port("CEB")
        ce_port("WEA")
        ce_port("WEB")
        ce_port("CSA")
        ce_port("CSB")
        ce_port("CEOUTA")
        ce_port("CEOUTB")
        rst_port("RSTA")
        rst_port("RSTB")
        data_port("BENA_N", 4)
        data_port("BENB_N", 4)
        data_port("ADA", 14)
        data_port("ADB", 14)
        data_port("DIA", 32)
        data_port("DIB", 32)
        output_port("DOA", i, 0,  32, False)
        output_port("DOB", i, 32, 32, False)
        output_port("ERRDECA", i, 64, 2, False)
        output_port("ERRDECB", i, 66, 2, True)
        print("    );")
    elif lram_prim == "PDPSC512K":
        next_data = get_next_data(i, 32+4)
        print("    PDPSC512K lram_{} (".format(i))
        clock_port("CLK")
        ce_port("CEW")
        ce_port("CER")
        ce_port("WE")
        ce_port("CSW")
        ce_port("CSR")
        rst_port("RSTR")
        data_port("BYTEEN_N", 4)
        data_port("ADW", 14)
        data_port("ADR", 14)
        data_port("DI", 32)
        output_port("DO", i, 0,  32, False)
        output_port("ERRDECA", i, 32, 2, False)
        output_port("ERRDECB", i, 34, 2, True)
        print("    );")
    elif lram_prim == "SP512K":
        next_data = get_next_data(i, 32+4)
        print("    SP512K lram_{} (".format(i))
        clock_port("CLK")
        ce_port("CE")
        ce_port("WE")
        ce_port("CS")
        ce_port("CEOUT")
        rst_port("RSTOUT")
        data_port("BYTEEN_N", 4)
        data_port("AD", 14)
        data_port("DI", 32)
        output_port("DO", i, 0,  32, False)
        output_port("ERRDECA", i, 32, 2, False)
        output_port("ERRDECB", i, 34, 2, True)
        print("    );")
    else:
        assert False
    data = next_data
print("    assign q = {};".format(data_bits(8)))
print("endmodule")