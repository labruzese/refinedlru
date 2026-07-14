From radium Require Import lang notation.
From refinedrust Require Import typing shims.
From vlru.vlru.generated Require Import generated_specs_vlru.

Set Default Proof Using "Type".

Section proof.
Context `{RRGS : !refinedrustGS Σ}.

Lemma KeyWrapperKascore_cmp_PartialEq_spec_subsumption_correct  : (KeyWrapperKascore_cmp_PartialEq_spec_subsumption).
Proof.
  unfold KeyWrapperKascore_cmp_PartialEq_spec_subsumption; solve_trait_incl_prelude.
  all: repeat liRStep; liShow.
  all: print_remaining_trait_goal.
  Unshelve.
  all: sidecond_solver.
  Unshelve.
  all: sidecond_hammer.
Qed.

End proof.
