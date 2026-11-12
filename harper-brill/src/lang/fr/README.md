# French models

The models in this folder are bundled only when the `fr` feature is enabled.

## Source

Both models are trained on [UD_French-GSD](https://github.com/UniversalDependencies/UD_French-GSD)
from the Universal Dependencies project, licensed under
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
The trained models are derived from those annotations and carry the same license.

## Retraining

Run `train.sh` from anywhere. It clones the treebank, builds `harper-cli` with training enabled,
and writes both models into this folder. `train.sh -h` lists its options.

## Bundled models

Trained with `train.sh` defaults: release `r2.19`, 100 epochs, candidate chance `0.02`,
on `fr_gsd-ud-train.conllu`, using 8 threads.

| Model   | Error rate before patches | At the last epoch | Training time |
| ------- | ------------------------- | ----------------- | ------------- |
| Tagger  | 3.90%                     | 2.72%             | about 3½ hours |
| Chunker | 13.95%                    | 10.28%            | about 1 hour  |

Error rates are measured on the training set. Because `-c` samples candidate patches at random,
retraining gives slightly different models.
