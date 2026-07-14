// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Primary model header
//
// This header should be included by all source files instantiating the design.
// The class here is then constructed to instantiate the design.
// See the Verilator manual for examples.

#ifndef VERILATED_VWIDE_PORTS_H_
#define VERILATED_VWIDE_PORTS_H_  // guard

#include "verilated.h"

class Vwide_ports__Syms;
class Vwide_ports___024root;

// This class is the main interface to the Verilated model
class alignas(VL_CACHE_LINE_BYTES) Vwide_ports VL_NOT_FINAL : public VerilatedModel {
  private:
    // Symbol table holding complete model state (owned by this class)
    Vwide_ports__Syms* const vlSymsp;

  public:

    // CONSTEXPR CAPABILITIES
    // Verilated with --trace?
    static constexpr bool traceCapable = false;

    // PORTS
    // The application code writes and reads these signals to
    // propagate new values into/out from the Verilated model.
    VL_INW(&__Vm_sig_input_u65,64,0,3);
    VL_INW(&__Vm_sig_input_u96,95,0,3);
    VL_INW(&__Vm_sig_input_u129,128,0,5);
    VL_INW(&__Vm_sig_input_u256,255,0,8);
    VL_INW(&__Vm_sig_input_i65,64,0,3);
    VL_INW(&__Vm_sig_input_i129,128,0,5);
    VL_OUTW(&__Vm_sig_output_u65,64,0,3);
    VL_OUTW(&__Vm_sig_output_u96,95,0,3);
    VL_OUTW(&__Vm_sig_output_u129,128,0,5);
    VL_OUTW(&__Vm_sig_output_u256,255,0,8);
    VL_OUTW(&__Vm_sig_output_i65,64,0,3);
    VL_OUTW(&__Vm_sig_output_i129,128,0,5);

    // ACCESSORS
    // The application code should use these methods to
    // propagate new values into/out from the Verilated model
    // instead of using signal variables directly.
    decltype(__Vm_sig_input_u65) input_u65() {return __Vm_sig_input_u65;}
    void input_u65(decltype(__Vm_sig_input_u65) v) {__Vm_sig_input_u65=v;}
    decltype(__Vm_sig_input_u96) input_u96() {return __Vm_sig_input_u96;}
    void input_u96(decltype(__Vm_sig_input_u96) v) {__Vm_sig_input_u96=v;}
    decltype(__Vm_sig_input_u129) input_u129() {return __Vm_sig_input_u129;}
    void input_u129(decltype(__Vm_sig_input_u129) v) {__Vm_sig_input_u129=v;}
    decltype(__Vm_sig_input_u256) input_u256() {return __Vm_sig_input_u256;}
    void input_u256(decltype(__Vm_sig_input_u256) v) {__Vm_sig_input_u256=v;}
    decltype(__Vm_sig_input_i65) input_i65() {return __Vm_sig_input_i65;}
    void input_i65(decltype(__Vm_sig_input_i65) v) {__Vm_sig_input_i65=v;}
    decltype(__Vm_sig_input_i129) input_i129() {return __Vm_sig_input_i129;}
    void input_i129(decltype(__Vm_sig_input_i129) v) {__Vm_sig_input_i129=v;}
    decltype(__Vm_sig_output_u65) output_u65() {return __Vm_sig_output_u65;}
    void output_u65(decltype(__Vm_sig_output_u65) v) {__Vm_sig_output_u65=v;}
    decltype(__Vm_sig_output_u96) output_u96() {return __Vm_sig_output_u96;}
    void output_u96(decltype(__Vm_sig_output_u96) v) {__Vm_sig_output_u96=v;}
    decltype(__Vm_sig_output_u129) output_u129() {return __Vm_sig_output_u129;}
    void output_u129(decltype(__Vm_sig_output_u129) v) {__Vm_sig_output_u129=v;}
    decltype(__Vm_sig_output_u256) output_u256() {return __Vm_sig_output_u256;}
    void output_u256(decltype(__Vm_sig_output_u256) v) {__Vm_sig_output_u256=v;}
    decltype(__Vm_sig_output_i65) output_i65() {return __Vm_sig_output_i65;}
    void output_i65(decltype(__Vm_sig_output_i65) v) {__Vm_sig_output_i65=v;}
    decltype(__Vm_sig_output_i129) output_i129() {return __Vm_sig_output_i129;}
    void output_i129(decltype(__Vm_sig_output_i129) v) {__Vm_sig_output_i129=v;}

    // CELLS
    // Public to allow access to /* verilator public */ items.
    // Otherwise the application code can consider these internals.

    // Root instance pointer to allow access to model internals,
    // including inlined /* verilator public_flat_* */ items.
    Vwide_ports___024root* const rootp;

    // CONSTRUCTORS
    /// Construct the model; called by application code
    /// If contextp is null, then the model will use the default global context
    /// If name is "", then makes a wrapper with a
    /// single model invisible with respect to DPI scope names.
    explicit Vwide_ports(VerilatedContext* contextp, const char* name = "TOP");
    explicit Vwide_ports(const char* name = "TOP");
    /// Destroy the model; called (often implicitly) by application code
    virtual ~Vwide_ports();
  private:
    VL_UNCOPYABLE(Vwide_ports);  ///< Copying not allowed

  public:
    // API METHODS
    /// Evaluate the model.  Application must call when inputs change.
    void eval() { eval_step(); }
    /// Evaluate when calling multiple units/models per time step.
    void eval_step();
    /// Evaluate at end of a timestep for tracing, when using eval_step().
    /// Application must call after all eval() and before time changes.
    void eval_end_step() {}
    /// Simulation complete, run final blocks.  Application must call on completion.
    void final();
    /// Are there scheduled events to handle?
    bool eventsPending();
    /// Returns time at next time slot. Aborts if !eventsPending()
    uint64_t nextTimeSlot();
    /// Trace signals in the model; called by application code
    void trace(VerilatedTraceBaseC* tfp, int levels, int options = 0) { contextp()->trace(tfp, levels, options); }
    /// Retrieve name of this model instance (as passed to constructor).
    const char* name() const;

    // Abstract methods from VerilatedModel
    const char* hierName() const override final;
    const char* modelName() const override final;
    unsigned threads() const override final;
    /// Prepare for cloning the model at the process level (e.g. fork in Linux)
    /// Release necessary resources. Called before cloning.
    void prepareClone() const;
    /// Re-init after cloning the model at the process level (e.g. fork in Linux)
    /// Re-allocate necessary resources. Called after cloning.
    void atClone() const;
  private:
    // Internal functions - trace registration
    void traceBaseModel(VerilatedTraceBaseC* tfp, int levels, int options);
};

#endif  // guard
