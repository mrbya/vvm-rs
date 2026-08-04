// ANCHOR: rtl
module line_adapter (
    input logic drive_enable,
    input logic drive_value,
    inout wire line,
    output wire sampled_line
);

assign line = drive_enable ? drive_value : 1'bz;
assign sampled_line = line;

endmodule
// ANCHOR_END: rtl
