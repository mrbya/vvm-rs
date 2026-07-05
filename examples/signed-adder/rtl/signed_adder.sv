module signed_adder (
    input  logic                    clk,
    input  logic signed [7:0]       lhs,
    input  logic signed [7:0]       rhs,
    output logic signed [8:0]       sum
);

always_ff @(posedge clk) begin
    sum <= $signed({lhs[7], lhs})
         + $signed({rhs[7], rhs});
end

endmodule
