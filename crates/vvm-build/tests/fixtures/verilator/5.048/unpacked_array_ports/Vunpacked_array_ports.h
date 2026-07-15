// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Primary model header

#ifndef VERILATED_VUNPACKED_ARRAY_PORTS_H_
#define VERILATED_VUNPACKED_ARRAY_PORTS_H_

#include "verilated.h"

class Vunpacked_array_ports__Syms;
class Vunpacked_array_ports___024root;

class alignas(VL_CACHE_LINE_BYTES) Vunpacked_array_ports VL_NOT_FINAL : public VerilatedModel {
  private:
    Vunpacked_array_ports__Syms* const vlSymsp;
  public:
    static constexpr bool traceCapable = false;
    VL_IN8(&clk,0,0);
    VL_OUT8(&first_flag_out,0,0);
    VL_OUT8(&last_flag_out,0,0);
    VL_OUT8(&bytes_left_out,7,0);
    VL_OUT8(&bytes_right_out,7,0);
    VL_OUT8(&wide_right_high_bit_out,0,0);
    VL_OUT(&wide_left_low_word_out,31,0);
    VlUnpacked<CData/*0:0*/, 4> &flags;
    VlUnpacked<CData/*0:0*/, 4> &flags_out;
    VlUnpacked<CData/*7:0*/, 4> &bytes;
    VlUnpacked<CData/*7:0*/, 4> &bytes_out;
    VlUnpacked<SData/*12:0*/, 3> &signed_values;
    VlUnpacked<SData/*12:0*/, 3> &signed_values_out;
    VlUnpacked<VlWide<5>/*128:0*/, 2> &wide_values;
    VlUnpacked<VlWide<5>/*128:0*/, 2> &wide_values_out;
    Vunpacked_array_ports___024root* const rootp;
    explicit Vunpacked_array_ports(VerilatedContext* contextp, const char* name = "TOP");
    explicit Vunpacked_array_ports(const char* name = "TOP");
    virtual ~Vunpacked_array_ports();
  private:
    VL_UNCOPYABLE(Vunpacked_array_ports);
  public:
    void eval() { eval_step(); }
    void eval_step();
    void eval_end_step() {}
    void final();
    bool eventsPending();
    uint64_t nextTimeSlot();
    void trace(VerilatedTraceBaseC* tfp, int levels, int options = 0) { contextp()->trace(tfp, levels, options); }
    const char* name() const;
    const char* hierName() const override final;
    const char* modelName() const override final;
    unsigned threads() const override final;
    void prepareClone() const;
    void atClone() const;
};

#endif
