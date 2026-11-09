---
title: "Domain-Agnostic Stroke Lesion Segmentation Using Physics-Constrained Synthetic Data"
authors: "Liam Chalcroft, Jenny Crinion, Cathy J Price, John Ashburner"
year: 2025
---

# Domain-Agnostic Stroke Lesion Segmentation Using Physics-Constrained Synthetic Data

**Authors:** Liam Chalcroft, Jenny Crinion, Cathy J Price, John Ashburner

## Abstract

Segmenting stroke lesions in MRI is challenging due to diverse acquisition protocols that limit model generalisability. In this work, we introduce two physics-constrained approaches to generate synthetic quantitative MRI (qMRI) images that improve segmentation robustness across heterogeneous domains. Our first method, qATLAS, trains a neural network to estimate qMRI maps from standard MPRAGE images, enabling the simulation of varied MRI sequences with realistic tissue contrasts. The second method, qSynth, synthesises qMRI maps directly from tissue labels using label-conditioned Gaussian mixture models, ensuring physical plausibility. Extensive experiments on multiple out-of-domain datasets show that both methods outperform a baseline UNet, with qSynth notably surpassing previous synthetic data approaches. These results highlight the promise of integrating MRI physics into synthetic data generation for robust, generalisable stroke lesion segmentation. Code is available at [https://github.com/liamchalcroft/qsynth](https://github.com/liamchalcroft/qsynth)

## Article

### Introduction

Segmenting brain pathologies in MRI is vital for clinical and research applications, yet remains challenging due to the variability of acquisition protocols across hospitals. Recent work shows that even for diffusion-weighted MRI -arguably the modality of choice in acute stroke -dedicated CNNs can equal human raters when trained on large, carefully curated cohorts [17]. While public datasets achieve high performance using standardised sequences (e.g., T1w, T2w, FLAIR) with similar parameters, clinical data rarely conforms to these ideal conditions . While public datasets achieve high performance using standardised sequences (e.g., T1w, T2w, FLAIR) with similar parameters, clinical data rarely conforms to these ideal conditions [21,6]. Existing domain adaptation methods either require prior target domain knowledge or many unlabelled images . Existing domain adaptation methods either require prior target domain knowledge or many unlabelled images [10,25], and synthetic approaches like SynthSeg , and synthetic approaches like SynthSeg [2] can produce unrealistic contrasts for heterogeneous stroke lesions can produce unrealistic contrasts for heterogeneous stroke lesions [1,8]..

To address these limitations, we propose generating synthetic images using quantitative MRI (qMRI) parameters and physics-based forward models to ensure physical plausibility. qMRI provides voxel-level tissue properties -proton density (PD), longitudinal relaxation rate (R 1 ), effective transverse relaxation rate (R * 2 ), and magnetisation transfer (MT) -that enable the simulation of diverse MRI sequences while preserving tissue characteristics. Since acquiring full qMRI data is time-consuming, recent deep learning methods estimate qMRI maps from standard sequences [4,24,3], making large-scale synthetic data generation feasible., making large-scale synthetic data generation feasible.

We introduce two methods for domain-agnostic stroke lesion segmentation. The first, qATLAS, trains a qMRI estimation model on MPRAGE images to augment the ATLAS dataset [16] with simulated sequences. The second, qSynth, extends previous synthetic approaches with simulated sequences. The second, qSynth, extends previous synthetic approaches [1,8] by sampling qMRI maps from intensity priors derived from real data, ensuring greater physical realism. By embedding MRI physics into data synthesis, our framework bridges the gap between synthetic and clinical data, thereby improving segmentation robustness across diverse imaging domains. by sampling qMRI maps from intensity priors derived from real data, ensuring greater physical realism. By embedding MRI physics into data synthesis, our framework bridges the gap between synthetic and clinical data, thereby improving segmentation robustness across diverse imaging domains.

### Methods

We propose two methods, qATLAS and qSynth, for domain-agnostic stroke lesion segmentation. Both leverage qMRI parameter maps to generate diverse, physicsconstrained training data, enhancing robustness and generalisability.

### qATLAS: Estimating qMRI from MPRAGE

qATLAS fits an nnU-Net [14] that predicts four qMRI maps (PD, R 1 , R * 2 , MT) from a single MPRAGE ‡ . Training used 51 subjects (22 healthy, 29 stroke; 80/20 split) with 3D-EPI ground-truth maps from hMRI that predicts four qMRI maps (PD, R 1 , R * 2 , MT) from a single MPRAGE ‡ . Training used 51 subjects (22 healthy, 29 stroke; 80/20 split) with 3D-EPI ground-truth maps from hMRI [22]. Thus any routine MPRAGE can be converted to qMRI and fed to our physics-based simulator.. Thus any routine MPRAGE can be converted to qMRI and fed to our physics-based simulator.

Diverse MPRAGE images were simulated using NiTorch with parameters: T R ∼ U(1.9, 2.5) s, T I ∼ U(0.6, 1.2) s, T E ∼ U(2, 4) ms, α ∼ U(5 • , 12 • ), and B 0 ∼ U(0.3, 7) T.

### Data Augmentation:

We applied standard augmentations (elastic/affine deformations, bias field, Gibbs ringing, Rician noise, and random cropping to 192 3 voxels) using MONAI [7]. Figure . Figure 1 illustrates examples of the augmented training data. illustrates examples of the augmented training data.

Model Architecture and Training: Our U-Net comprises five encoder stages (channels: 24, 48, 96, 192, 384), each with two residual units, using GELU activations [13], instance normalisation , instance normalisation [23], linear upsampling, and 0.1 dropout. The network outputs four channels corresponding to PD, R 1 , R * 2 , and MT. To enforce positivity, PD, R 1 , and R * 2 are computed as the exponential of their raw outputs, while MT is computed as 100 times the sigmoid of its raw output., linear upsampling, and 0.1 dropout. The network outputs four channels corresponding to PD, R 1 , R * 2 , and MT. To enforce positivity, PD, R 1 , and R * 2 are computed as the exponential of their raw outputs, while MT is computed as 100 times the sigmoid of its raw output.

Training was performed over 200,000 iterations (batch size 1) using AdamW (lr=10 -4 , β 1 = 0.9, β 2 = 0.999, weight decay=0.01). An initial 20,000 iterations

$$
‡ Estimating R *
$$

2 from one T1 is ill-posed; errors mainly affect simulated T2w/FLAIR.

Fig. 1: Examples of training data for qMRI parameter map prediction in the qATLAS method. used L2 loss; thereafter, we employed a combined loss L = L1(y, ŷ) + L2(y, ŷ) + L1(∇y, ∇ŷ) + L2(∇y, ∇ŷ) augmented by a perceptual LPIPS loss : Examples of training data for qMRI parameter map prediction in the qATLAS method. used L2 loss; thereafter, we employed a combined loss L = L1(y, ŷ) + L2(y, ŷ) + L1(∇y, ∇ŷ) + L2(∇y, ∇ŷ) augmented by a perceptual LPIPS loss [27] (weight 0.1) using features from a pretrained Med3D encoder (weight 0.1) using features from a pretrained Med3D encoder [9]..

Figure 2 displays examples of predicted qMRI maps from input MPRAGE images in the ATLAS dataset displays examples of predicted qMRI maps from input MPRAGE images in the ATLAS dataset [16]. Simulation of MRI Sequences: We used the estimated qMRI maps from the ATLAS dataset to simulate diverse MRI sequences via a physics-based generative model (see Section 2.3), thereby creating the qATLAS dataset for segmentation training.. Simulation of MRI Sequences: We used the estimated qMRI maps from the ATLAS dataset to simulate diverse MRI sequences via a physics-based generative model (see Section 2.3), thereby creating the qATLAS dataset for segmentation training.

### qSynth: Synthesising qMRI Maps from Tissue Labels

The qSynth method generates synthetic qMRI maps directly from segmentation labels using label-conditioned Gaussian Mixture Models (GMMs). We first define prior distributions for each qMRI parameter (PD, R 1 , R * 2 , MT) for different tissue types (gray matter, white matter, cerebrospinal fluid, and lesions). These priors are estimated from the population used in qATLAS (for anatomy or pathology where real data is not available, it is feasible that parameters could be estimated based on values reported in literature). For each voxel in a segmentation label, the corresponding qMRI parameters are sampled from the appropriate prior, thereby producing a synthetic qMRI map that reflects realistic tissue properties. Healthy tissue maps are obtained using the Multibrain SPM toolbox [5], and lesion masks are generated by overlaying random binary lesion maps onto these healthy tissue maps (see , and lesion masks are generated by overlaying random binary lesion maps onto these healthy tissue maps (see [8] for further details). for further details).

Simulation of MRI Sequences: The synthetic qMRI maps are then used with our physics-based generative model (Section 2.3) to simulate diverse MRI sequences, resulting in the qSynth dataset for training an alternative segmentation model.

### Physics-Based Generative Model

Both qATLAS and qSynth use a physics-based generative model to simulate realistic MRI images from qMRI parameter maps via standard signal equations. We simulate several common MRI sequences, including Fast Spin-Echo (FSE), Gradient-Echo (GRE), Fluid-Attenuated Inversion Recovery (FLAIR), and Magnetisation-Prepared Rapid Gradient Echo (MPRAGE).

For example, the FSE signal is computed as:

$$
S FSE = B 1 PD 1 -e -R1T R e -R2T E ,
$$

where B 1 is the receive field strength, PD is the proton density, R 1 and R 2 (≈ R2 * ) are the relaxation rates, and T R and T E are the repetition and echo times, respectively. Similar equations are used for GRE, FLAIR, and MPRAGE sequences. Acquisition parameters (e.g., T R , T E , T I , T X , T D , and α) were sampled from physically plausible distributions; please refer to our code repository § for the complete sampling details.

To mimic realistic MRI data, we add Rician noise by perturbing the simulated signal S MRI with independent Gaussian noise on the real and imaginary components:

$$
S noisy = (S MRI + n r ) 2 + n 2 i , n r , n i ∼ N (0, σ 2 ).
$$

All simulations and noise additions were implemented using the NiTorch library ¶ . Additional augmentations -such as elastic deformations, bias field variations, axis flips, Gaussian noise, low-resolution reslicing, and random cropping to 192 × 192 × 192 voxels -were applied using MONAI [7]..

### Segmentation Model Training

We trained nnUNet-based segmentation models [14] on four settings: (i) real MPRAGE images from the ATLAS dataset (baseline), (ii) the qATLAS dataset (from MPRAGE-derived qMRI maps), (iii) a synthetic data model using the public Synth method on four settings: (i) real MPRAGE images from the ATLAS dataset (baseline), (ii) the qATLAS dataset (from MPRAGE-derived qMRI maps), (iii) a synthetic data model using the public Synth method [8], and (iv) the qSynth method. The qSynth method differs from Synth in that Synth directly samples the random tissue intensities, while qSynth uses intensity priors to first synthesise qMRI parameter maps, which may then be used to generate realistic tissue intensities via random variations of real forward models. For Synth and qSynth, we additionally evaluated models trained on a mixture of synthetic and real ATLAS images., and (iv) the qSynth method. The qSynth method differs from Synth in that Synth directly samples the random tissue intensities, while qSynth uses intensity priors to first synthesise qMRI parameter maps, which may then be used to generate realistic tissue intensities via random variations of real forward models. For Synth and qSynth, we additionally evaluated models trained on a mixture of synthetic and real ATLAS images.

Model Architecture and Training Details: All models use PReLU activations [12] with one residual unit per block. The qATLAS model performs binary segmentation (background vs. stroke lesion), while the qSynth model predicts additional healthy tissue classes (gray matter, white matter, GM/WM partial volume, and cerebrospinal fluid). Baseline and qATLAS models were trained on the ATLAS dataset (N=419/105/131 train/validation/test), whereas Synth and qSynth models used ATLAS lesion labels combined with healthy tissue labels from OASIS-3 (N=2579/100 train/validation) as detailed in with one residual unit per block. The qATLAS model performs binary segmentation (background vs. stroke lesion), while the qSynth model predicts additional healthy tissue classes (gray matter, white matter, GM/WM partial volume, and cerebrospinal fluid). Baseline and qATLAS models were trained on the ATLAS dataset (N=419/105/131 train/validation/test), whereas Synth and qSynth models used ATLAS lesion labels combined with healthy tissue labels from OASIS-3 (N=2579/100 train/validation) as detailed in [8]..

All models were optimised using a combined Dice and cross-entropy loss with the AdamW optimiser [19] (lr=10 -4 , β 1 = 0.9, β 2 = 0.999, weight decay=0.01) and a learning rate scheduler η n = η 0 (1 -n N ) 0.9 . Training was run for 700,000 iterations with a batch size of 1. (lr=10 -4 , β 1 = 0.9, β 2 = 0.999, weight decay=0.01) and a learning rate scheduler η n = η 0 (1 -n N ) 0.9 . Training was run for 700,000 iterations with a batch size of 1.

### Experiments

We evaluated segmentation models on four datasets: ATLAS, comprising 131 subjects (isotropic MPRAGE) [16] for in-domain evaluation; ARC, with 229 subjects (T1w, T2w, FLAIR) for in-domain evaluation; ARC, with 229 subjects (T1w, T2w, FLAIR) [11,15] for out-of-domain testing; PLORAS, consisting of 406 subjects (106 T2w, 300 FLAIR) from various UK hospitals; and ISLES 2015, including 28 subjects (T1w, T2w, FLAIR, DWI) for out-of-domain testing; PLORAS, consisting of 406 subjects (106 T2w, 300 FLAIR) from various UK hospitals; and ISLES 2015, including 28 subjects (T1w, T2w, FLAIR, DWI) [20] with acute lesions. with acute lesions.

All scans were resliced to 1 mm, histogram-normalised and z-scored, then segmented with a 192 3 sliding window (50 % overlap, Gaussian blending) and 8flip TTA [26]. For multi-modal sets (ARC, ISLES15) we averaged per-voxel logits across contrasts before soft-max and post-processing (reported as 'Ensemble'). Dice and HD95 (95-percentile Hausdorff, mm) were computed on predictions/GT padded to 256 3 ; only the lesion channel is reported for Synth/qSynth. Dataset caveat. Synth/qSynth are trained on a much larger multi-site cohort with multi-class labels, whereas qATLAS and the baseline use ATLAS only (binary masks). Thus absolute score differences combine physics, training-set size, label granularity and pre-stripped skulls in ISLES15; isolating each factor is left for future work.. For multi-modal sets (ARC, ISLES15) we averaged per-voxel logits across contrasts before soft-max and post-processing (reported as 'Ensemble'). Dice and HD95 (95-percentile Hausdorff, mm) were computed on predictions/GT padded to 256 3 ; only the lesion channel is reported for Synth/qSynth. Dataset caveat. Synth/qSynth are trained on a much larger multi-site cohort with multi-class labels, whereas qATLAS and the baseline use ATLAS only (binary masks). Thus absolute score differences combine physics, training-set size, label granularity and pre-stripped skulls in ISLES15; isolating each factor is left for future work.

### Results

Fig. 3: Dice metric performance for all datasets.: Dice metric performance for all datasets.

### ATLAS:

The ATLAS test set is considered in-domain. The baseline model is expected to perform optimally, while the qMRI generator model must accurately reproduce MPRAGE details to match this performance. As shown in Tables 1 & 2 and Figure 3, the baseline outperformed the qMRI-based methods, with no statistically significant differences except between Synth and qSynth. Notably, qSynth significantly outperformed Synth, indicating that the physicsconstrained pipeline reduces domain shift between simulated and real data. ARC: The ARC dataset exhibits moderate domain shift in the T1w channel and larger shifts in T2 and FLAIR. As shown in Tables 1 & 2 and Figure , the baseline outperformed the qMRI-based methods, with no statistically significant differences except between Synth and qSynth. Notably, qSynth significantly outperformed Synth, indicating that the physicsconstrained pipeline reduces domain shift between simulated and real data. ARC: The ARC dataset exhibits moderate domain shift in the T1w channel and larger shifts in T2 and FLAIR. As shown in Tables 1 & 2 and Figure 3, qATLAS underperformed relative to the baseline (though not significantly). For T1w, qSynth initially lagged behind Synth but reached comparable performance when real data was integrated; in T2w, qSynth outperformed all models, and in Table , qATLAS underperformed relative to the baseline (though not significantly). For T1w, qSynth initially lagged behind Synth but reached comparable performance when real data was integrated; in T2w, qSynth outperformed all models, and in Table 1: Median Dice (top line) and 95 % CI (tiny line). Best, second, and third best are bold, underlined, and italic.: Median Dice (top line) and 95 % CI (tiny line). Best, second, and third best are bold, underlined, and italic.

### Conclusion

We proposed two physics-aware generators for stroke-lesion segmentation training: qATLAS, which derives qMRI maps from a single MPRAGE, and qSynth, which samples qMRI from learned intensity priors. qATLAS boosts T1-weighted performance but degrades on contrasts dominated by non-R 1 parameters; qSynth delivers the most consistent cross-domain gains, yet both remain below fully in-domain models. Future work should explore domain adaptation techniques or domain-conditioning via hypernetworks to further close this gap. Additionally, our framework can be extended to other tasks (e.g., registration, superresolution), other pathologies (e.g. glioblastoma), and other anatomies (e.g., cardiac, abdomen). We also plan to integrate recent advances in more realistic lesion simulations [18] to further enhance synthetic data fidelity. Overall, embedding MRI physics in data generation represents a promising step toward robust, generalisable segmentation in diverse clinical scenarios. to further enhance synthetic data fidelity. Overall, embedding MRI physics in data generation represents a promising step toward robust, generalisable segmentation in diverse clinical scenarios.

> 2 Fig. 2 : Fig. 2: Examples of predicted qMRI parameter maps from input MPRAGE images of the ATLAS dataset in the qATLAS method.

## References

- B Billot, S Cerri, K V Leemput. [Joint segmentation of multiple sclerosis lesions and brain anatomy in MRI scans of any contrast and resolution with CNNs](https://doi.org/10.1109/isbi48211.2021.9434127). 2021 IEEE 18th International Symposium on Biomedical Imaging (ISBI). 2021
- B Billot, D N Greve, O Puonti. [SynthSeg: Segmentation of brain MRI scans of any contrast and resolution without retraining](https://doi.org/10.1016/j.media.2023.102789). Medical Image Analysis. 2023
- Unsupervised Heteromodal Physics-Informed Representation of MRI Data: Tackling Data Harmonisation, Imputation and Domain Shift. 2023
- Acquisition-invariant brain mri segmentation with informative uncertainties. 2021
- Flexible Bayesian Modelling for Nonlinear Image Registration. 2020
- M Brzus, A D Boes, J Bruss, H J Johnson. [Leveraging high-quality research data for ischemic stroke lesion segmentation on clinical data](https://doi.org/10.1109/ISBI53787.2023.10230775). 2023 IEEE 20th International Symposium on Biomedical Imaging (ISBI). 2023
- Monai: An open-source framework for deep learning in healthcare. 2022
- Synthetic data for robust stroke segmentation. 2024
- Med3d: Transfer learning for 3d medical image analysis. 2019
- R Dorent, A Kujawa, M Ivory. [Crossmoda 2021 challenge: Benchmark of cross-modality domain adaptation techniques for vestibular schwannoma and cochlea segmentation](https://doi.org/10.1016/j.media.2022.102628). Medical Image Analysis. 2023
- M Gibson, R Newman-Norlund, L Bonilha. [The Aphasia Recovery Cohort, an open-source chronic stroke repository](https://doi.org/10.1038/s41597-024-03819-7). Scientific Data. 2024
- Delving deep into rectifiers: Surpassing human-level performance on ImageNet classification. 2015
- Gaussian error linear units (gelus. 2023
- F Isensee, P F Jaeger, S A A Kohl. [nnU-Net: a self-configuring method for deep learning-based biomedical image segmentation](https://doi.org/10.1038/s41592-020-01008-z). Nature Methods. 2020
- L Johnson, R Newman-Norlund, A Teghipco. [Progressive lesion necrosis is related to increasing aphasia severity in chronic stroke](https://doi.org/10.1016/j.nicl.2024.103566). Neu-roImage: Clinical. 2024
- S L Liew, B P Lo, M R Donnelly. [A large, curated, opensource stroke neuroimaging dataset to improve lesion segmentation algorithms](https://doi.org/10.1038/s41597-022-01401-7). Scientific Data. 2022
- C F Liu, J Hsu, X Xu. [Deep learning-based detection and segmentation of diffusion abnormalities in acute ischemic stroke](https://doi.org/10.1038/s43856-021-00062-8). Communications Medicine. 2021
- Unraveling normal anatomy via fluiddriven anomaly randomization. 2025
- Decoupled weight decay regularization. 2019
- O Maier, B H Menze, J Der Gablentz. [ISLES 2015 -a public evaluation benchmark for ischemic stroke lesion segmentation from multispectral MRI](https://doi.org/10.1016/j.media.2016.07.009). Medical Image Analysis. 2017
- A robust ensemble algorithm for ischemic stroke lesion segmentation: Generalizability and clinical utility beyond the isles challenge. 2024
- K Tabelow, E Balteau, J Ashburner. [hmri -a toolbox for quantitative mri in neuroscience and clinical research](https://doi.org/10.1016/j.neuroimage.2019.01.029). NeuroImage. 2019
- Instance normalization: The missing ingredient for fast stylization. 2017
- Unsupervised learning of mri tissue properties using mri physics models. 2021
- Tent: Fully test-time adaptation by entropy minimization. 2021
- Automatic Brain Tumor Segmentation Using Convolutional Neural Networks with Test-Time Augmentation. 2019
- The unreasonable effectiveness of deep features as a perceptual metric. 2018
