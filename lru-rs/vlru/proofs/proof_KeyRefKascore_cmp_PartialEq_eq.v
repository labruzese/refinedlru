From radium Require Import lang notation.
From refinedrust Require Import typing shims.
From vlru.vlru.generated Require Import generated_code_vlru generated_specs_vlru generated_template_KeyRefKascore_cmp_PartialEq_eq.

Set Default Proof Using "Type".

Section proof.
Context `{RRGS : !refinedrustGS Σ}.

Lemma KeyRefKascore_cmp_PartialEq_eq_proof (π : thread_id) :
  KeyRefKascore_cmp_PartialEq_eq_lemma π.
Proof.
  KeyRefKascore_cmp_PartialEq_eq_prelude.

  rep <-! liRStep; liShow.

  all: print_remaining_goal.
  Unshelve. all: sidecond_solver.
  Unshelve. all: sidecond_hammer.
  Unshelve. all: print_remaining_sidecond.
Qed.
End proof.
