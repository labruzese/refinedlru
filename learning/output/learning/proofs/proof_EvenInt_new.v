From radium Require Import lang notation.
From refinedrust Require Import typing shims.
From learning.learning.generated Require Import generated_code_learning generated_specs_learning generated_template_EvenInt_new.

Set Default Proof Using "Type".

Section proof.
Context `{RRGS : !refinedrustGS Σ}.

Lemma EvenInt_new_proof (π : thread_id) :
  EvenInt_new_lemma π.
Proof.
  EvenInt_new_prelude.

  rep <-! liRStep; liShow.

  all: print_remaining_goal.
  Unshelve. all: sidecond_solver.
  Unshelve. all: sidecond_hammer.
  Unshelve. all: print_remaining_sidecond.
Qed.
End proof.
