module multi_clock_counter (
    input logic core_clk,
    input logic peripheral_clk,
    input logic reset_n,
    input logic peripheral_enable,
    output logic [7:0] peripheral_count,
    output logic [7:0] core_sample
);
always_ff @(posedge peripheral_clk or negedge reset_n) begin
    if (!reset_n) peripheral_count <= '0;
    else if (peripheral_enable) peripheral_count <= peripheral_count + 1'b1;
end
always_ff @(posedge core_clk or negedge reset_n) begin
    if (!reset_n) core_sample <= '0;
    else core_sample <= peripheral_count;
end
endmodule
