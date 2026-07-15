From radium Require Import lang notation.
From refinedrust Require Import typing shims.
From lru.verification.vlru.generated Require Import generated_code_vlru generated_specs_vlru generated_template_drop_glue_LruEntry.

Set Default Proof Using "Type".

Section proof.
Context `{RRGS : !refinedrustGS Σ}.

Lemma drop_glue_LruEntry_proof (π : thread_id) :
  drop_glue_LruEntry_lemma π.
Proof.
  drop_glue_LruEntry_prelude.

  rep <-! liRStep; liShow.

  all: print_remaining_goal.
  Unshelve. all: sidecond_solver.
  Unshelve. all: sidecond_hammer.
  Unshelve. all: print_remaining_sidecond.
Qed.
End proof.
