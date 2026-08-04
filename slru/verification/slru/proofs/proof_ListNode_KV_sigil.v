From radium Require Import lang notation.
From refinedrust Require Import typing shims.
From slru.verification.slru.generated Require Import generated_code_slru generated_specs_slru generated_template_ListNode_KV_sigil.

Set Default Proof Using "Type".

Section proof.
Context `{RRGS : !refinedrustGS Σ}.

Lemma ListNode_KV_sigil_proof (π : thread_id) :
  ListNode_KV_sigil_lemma π.
Proof.
  ListNode_KV_sigil_prelude.

  rep <-! liRStep; liShow.

  all: print_remaining_goal.
  Unshelve. all: sidecond_solver.
  Unshelve. all: sidecond_hammer.
  Unshelve. all: print_remaining_sidecond.
Qed.
End proof.
