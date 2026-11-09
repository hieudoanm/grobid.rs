---
title: "Machine Learning for Biomedical Imaging Synthetic Data for Robust Stroke Segmentation"
authors: "Liam Chalcroft, Ioannis Pappas, Cathy J Price, John Ashburner"
doi: "10.59275/j.melba.2025-f3g6"
doi_link: "https://doi.org/10.59275/j.melba.2025-f3g6"
---

# Machine Learning for Biomedical Imaging Synthetic Data for Robust Stroke Segmentation

**Authors:** Liam Chalcroft, Ioannis Pappas, Cathy J Price, John Ashburner

## Abstract

Current deep learning-based approaches to lesion segmentation in neuroimaging often depend on high-resolution images and extensive annotated data, limiting clinical applicability. This paper introduces a novel synthetic data framework tailored for stroke lesion segmentation, expanding the SynthSeg methodology to incorporate lesion-specific augmentations that simulate diverse pathological features. Using a modified nnUNet architecture, our approach trains models with label maps from healthy and stroke datasets, facilitating segmentation across both normal and pathological tissue without reliance on specific sequence-based training. Our method achieves robust out-of-domain performance where conventional approaches fail, with in-domain performance of 48.2% Dice compared to 57.5% for conventional training. Crucially, even with oracle knowledge of the optimal domain adaptation method -an unrealistic scenario in practice -conventionally-trained models cannot match our synthetic approach in out-of-domain settings. The framework demonstrates that synthetic pre-training provides fundamental robustness unachievable through test-time adaptation alone. Our approach reduces reliance on domain-specific training data and helps bridge the gap between research-grade and clinical scans to improve clinical stroke neuroimaging workflows.

## Article

### Introduction

S emantic segmentation is a critical component of neuroimaging pipelines, enabling precise quantification of anatomical structures and lesions for applications like tracking disease progression and planning treatments. In research settings, segmentation labels are typically derived from high-quality, standardised structural scans (e.g., MPRAGE) that benefit from consistent field-of-view, spacing, orientation, and minimal artifacts. In contrast, clinical scans often exhibit significant variability in these factors, which can severely impact deep learning model performance. Consequently, models trained on homogeneous researchgrade data may not generalise well to the diverse, lowerquality images encountered in clinical practice.

Both traditional probabilistic methods (Ashburner and Friston, 2005) and modern deep discriminative methods and modern deep discriminative methods (Isensee et al., 2020) require prior information to be provided for a given sequence -in the case of a traditional method this may be an atlas or template, and in modern methods this would come in the form of training data. Atlas-based methods build a template of the anatomical structure of the brain, which may be deformed into alignment with a new subject to assign voxel-wise anatomical classes. This is proven to be robust for delineating healthy structure even with shifts in contrast require prior information to be provided for a given sequence -in the case of a traditional method this may be an atlas or template, and in modern methods this would come in the form of training data. Atlas-based methods build a template of the anatomical structure of the brain, which may be deformed into alignment with a new subject to assign voxel-wise anatomical classes. This is proven to be robust for delineating healthy structure even with shifts in contrast (Puonti et al., 2016), however it is non-trivial to include classes of pathology such as stroke within such a model, due to the inherent heterogeneity in location and geometric properties. In the context of generative models, lesions may be included in the form of anomaly detection as demonstrated in , however it is non-trivial to include classes of pathology such as stroke within such a model, due to the inherent heterogeneity in location and geometric properties. In the context of generative models, lesions may be included in the form of anomaly detection as demonstrated in Seghier et al. (2008). This method is not however directly attempting to label the pathology, and (by design) will label physiological changes such as ventricular enlargement in addition to the responsible infarct.. This method is not however directly attempting to label the pathology, and (by design) will label physiological changes such as ventricular enlargement in addition to the responsible infarct.

Deep discriminative models trained using supervised learning have been able to reach human-level performance when tested in-domain on large datasets for a variety of brain pathologies and imaging modalities (Baid et al., 2021;de la Rosa et al., 2024). There is still however a significant gap when trying to translate these models to clinical data, where each hospital is likely to vary both in the scanning equipment used and the choice of imaging sequences . There is still however a significant gap when trying to translate these models to clinical data, where each hospital is likely to vary both in the scanning equipment used and the choice of imaging sequences (Nguyen et al., 2024). This poses a significant challenge to the adoption of deep learning for automating the labelling of clinical data, which could greatly help to accelerate the translation of modern research in stroke prognosis . This poses a significant challenge to the adoption of deep learning for automating the labelling of clinical data, which could greatly help to accelerate the translation of modern research in stroke prognosis (Loughnan et al., 2019)..

To extend such methods to the open-ended domain of clinical scans, models often need to perform on sequences for which no training data may be available. To this end, domain randomisation via synthetic data has been shown to give impressive results for healthy brain parcellation in SynthSeg (Billot et al., 2023). In this method, a set of ground truth healthy tissues are used to generate synthetic images, under the assumption that each tissue class' intensity distribution should roughly follow a Gaussian. By assigning random Gaussian distributions to each class, a deep learning model can learn to extract shape information for parcellation in a way that is invariant to the input image's relative tissue contrast, hence allowing the model to be used on any sequence at test-time, without training data or prior knowledge of the sequence. This method of training with synthetic data has since been extended to tasks such as image registration . In this method, a set of ground truth healthy tissues are used to generate synthetic images, under the assumption that each tissue class' intensity distribution should roughly follow a Gaussian. By assigning random Gaussian distributions to each class, a deep learning model can learn to extract shape information for parcellation in a way that is invariant to the input image's relative tissue contrast, hence allowing the model to be used on any sequence at test-time, without training data or prior knowledge of the sequence. This method of training with synthetic data has since been extended to tasks such as image registration (Hoffmann et al., 2022(Hoffmann et al., , 2023(Hoffmann et al., , 2024)), image super-resolution , image super-resolution (Iglesias et al., 2021(Iglesias et al., , 2023)), surface estimation , surface estimation (Gopinath et al., 2023(Gopinath et al., , 2024a) ) and vascular segmentation and vascular segmentation (Chollet et al., 2024). A comprehensive overview is available in . A comprehensive overview is available in Gopinath et al. (2024b)..

An additional benefit of this method is that the 'forward model' of creating an MRI (or CT) image from tissues of different physical properties is a perfect 1:1 mapping to the 'inverse model' of labeling the tissues (i.e. segmentation) from the acquired image. In structures that exhibit a large amount of inter-rater variability, this is likely to help prevent a model from imitating under-or over-segmentation from imperfect ground truths -the images segmented are generated from the corresponding segmentation labels and so labels will always be a consistent method of segmentation.

Prior work to SynthSeg demonstrated the potential of encoding anatomical priors in a neural network through pre-training with unpaired parcellation labels (Dalca et al., 2018). Such methods face significantly larger challenges when applied to the heterogeneous shape and spatial distribution of lesions. In healthy parcellation, anatomical structures have consistent positions across individuals (e.g., the brainstem reliably appears in the same region of the brain), a regularity that has motivated atlas-based approaches to parcellation. In contrast, lesions are highly variable across individuals -not only in number and size but also in their spatial distribution. Unlike anatomical structures, lesions cannot be reliably mapped to a specific location or shape within an atlas. Although the exact site of lesion initiation is often influenced by the brain's vascular architecturemeaning that certain regions are statistically more susceptible to stroke due to the location of large blood vessels -the resulting lesion's size, shape, and spread are highly variable. Multiple sclerosis serves as an exception, with lesions that are somewhat predictable in their white matter localisation . Such methods face significantly larger challenges when applied to the heterogeneous shape and spatial distribution of lesions. In healthy parcellation, anatomical structures have consistent positions across individuals (e.g., the brainstem reliably appears in the same region of the brain), a regularity that has motivated atlas-based approaches to parcellation. In contrast, lesions are highly variable across individuals -not only in number and size but also in their spatial distribution. Unlike anatomical structures, lesions cannot be reliably mapped to a specific location or shape within an atlas. Although the exact site of lesion initiation is often influenced by the brain's vascular architecturemeaning that certain regions are statistically more susceptible to stroke due to the location of large blood vessels -the resulting lesion's size, shape, and spread are highly variable. Multiple sclerosis serves as an exception, with lesions that are somewhat predictable in their white matter localisation (Lassmann, 2018), enabling modelling through synthetic deep learning frameworks , enabling modelling through synthetic deep learning frameworks (Billot et al., 2021;Laso et al., 2024) and traditional probabilistic models and traditional probabilistic models (Cerri et al., 2021). More recently, . More recently, Liu et al. (2024) has shown promising results by training a SynthSeg-like model on lesion labels from various pathologies, providing a foundation for fine-tuning on multiple downstream datasets. has shown promising results by training a SynthSeg-like model on lesion labels from various pathologies, providing a foundation for fine-tuning on multiple downstream datasets.

Robust open-domain stroke segmentation remains an unsolved challenge. Most domain-specific frameworks for stroke lesion segmentation are targeted towards lesionpasting (Zhang et al., 2021;Dai et al., 2022;Basaran et al., 2023), aiming to augment the anatomical variety without making any attempt to augment the variety in image contrasts. Likewise, label-conditioned generative models such as TumorGAN , aiming to augment the anatomical variety without making any attempt to augment the variety in image contrasts. Likewise, label-conditioned generative models such as TumorGAN (Li et al., 2020) can similarly only generate new lesioned brains within the learned distribution of image intensities. None of these works approach the task of robustness to shifts in image appearance, instead focusing on shape-related augmentation. can similarly only generate new lesioned brains within the learned distribution of image intensities. None of these works approach the task of robustness to shifts in image appearance, instead focusing on shape-related augmentation.

In our work, we extend the SynthSeg framework to the task of stroke lesion segmentation via a novel lesionpasting method that better simulates variety in lesion appearance. Our hybrid approach trades a statistically significant 9.3% median Dice reduction in-domain (57.5% vs 48.2%, p<0.001) for improved out-of-domain robustness. Crucially, we demonstrate that even with oracle knowledge of the optimal domain adaptation method, conventional training cannot match our synthetic approach in out-ofdomain scenarios. We validate this on a comprehensive range of lesion datasets with a wide distribution of image characteristics and lesion physiology. To assist in widespread evaluation of this framework, we release PyTorch training code/weights, and a MATLAB toolbox for SPM to reduce the barrier to clinical adoption.

### Methods

### Terminology and Notation

For clarity, we define the key terms used throughout this work:

• TTA (Test-Time Augmentation): A procedure at infer-ence where multiple augmented versions (here generated by flips) of an input are processed and their predictions averaged to improve robustness.

• DA (Domain Adaptation): Techniques applied at test time to adapt a trained model to new data distributions.

• Oracle DA: The hypothetical best-case scenario where the optimal DA method is known a priori for each dataset/modality combination.

### Synthetic Data Generation Framework

Rationale. Our goal is to create a large, diverse and perfectly labelled training set without the labour of voxelwise annotation. We therefore generate paired image-label volumes by composing healthy tissue maps with realistically shaped stroke lesions, followed by intensity synthesis and heavy image-quality augmentation (Fig. 2).).

(i) Healthy-tissue label bank. Instead of the 100+ FreeSurfer classes used by SynthSeg, we adopt the nine posterior tissue maps produced by MultiBrain (Brudfors et al., 2020). This reduces memory usage, speeds up sampling and still retains the GM/WM/CSF boundaries that matter for lesion realism (Fig. . This reduces memory usage, speeds up sampling and still retains the GM/WM/CSF boundaries that matter for lesion realism (Fig. 1).).

(ii) Lesion Copy-Paste (Lesion-CP). We extend Soft-CP (Dai et al., 2022) with random dilate/erode 'feathering' and a spatially varying bias-field multiplier (MONAI Random Bias Field with random dilate/erode 'feathering' and a spatially varying bias-field multiplier (MONAI Random Bias Field (Cardoso et al., 2022)) to mimic penumbral intensity fall-off ) to mimic penumbral intensity fall-off (Middleton et al., 2024)..

(iii) Intensity sampling. Each tissue class is assigned µ ∼ U(0, 255), σ ∼ U(0, 16) and Gaussian blur FWHM ∼ U(0, 2). For stroke lesions we modulate the copy-pasted mask with the bias field to create intra-lesion heterogeneity. We sample from a single Gaussian distribution, with the implications of this choice examined via Hartigan's dip-test in Appendix A.

(iv) Image-quality augmentations. Table 1 summarises every random transform (bias field, affine, elastic, skull-strip imperfection, noise, resolution, motion, contrast, etc.). Future work will explore mixture-of-Gaussians modelling summarises every random transform (bias field, affine, elastic, skull-strip imperfection, noise, resolution, motion, contrast, etc.). Future work will explore mixture-of-Gaussians modelling (Ashburner and Friston, 2005) to represent lesions that are simultaneously hyper-and hypo-intense. to represent lesions that are simultaneously hyper-and hypo-intense.

With this synthetic data generation pipeline established, we now describe the datasets, network architecture, and training procedures used in our experiments.

### Training Data and Sampling

Healthy maps: OASIS-3 (N=2 679, 2 579/100 train/val) with MultiBrain segmentation, all warped to ICBM space. Lesion masks: ATLAS (N=655, 419/105/131 train/val/test) aligned to the same space. During sampling we paste one random ATLAS lesion onto one random OASIS subject and on-the-fly augment as above. Healthy tissue (N>2000) Lesion binary (N>400) Random label selection Lesion paste + INU augment Tissue intensity assignment L SEG (ŷ, y) UNet Figure 2: Schematic overview of the data generation process. Lesions are sampled from a template-normalised bank of lesion binary masks, and healthy tissue maps are sampled from a template-normalised bank of MultiBrain segmentations.

Pasting of lesions onto healthy tissue maps is performed using a spatially varying lesion intensity to simulate penumbra. Tissue intensities may then be sampled from Gaussian distributions and image-label pairs used to train a segmentation model in a supervised manner.

### Network Architecture and Optimisation

Backbone. 3D U-Net (nnUNet template) with six levels (16 → 32 → 64 → 128 → 320 channels), PReLU activations and one residual unit per block (Isensee et al., 2024). Comparison model. WMH-SynthSeg . Comparison model. WMH-SynthSeg (Laso et al., 2024;Fischl, 2012) is included as an "off-the-shelf" robust-lesion baseline, but its training labels target small, periventricular WMHs and differ substantially from large-cortical stroke lesions. is included as an "off-the-shelf" robust-lesion baseline, but its training labels target small, periventricular WMHs and differ substantially from large-cortical stroke lesions.

### Domain Adaptation Methods

At test time we evaluate a diverse set of unsupervised domain-adaptation (DA) techniques to determine: (i) whether the Baseline model can, under an oracle choice of DA method, match the performance of the DA-free Synth model, and (ii) whether Synth's inherent robustness offers a better starting point for DA on truly unseen data.

We tested six DA methods (TTA, DAE, TENT, PL, UPL, DPL) and report the best-performing configuration for each modality/dataset combination as Oracle DA in Tables 2345. This represents an upper bound on baseline performance, as in practice one cannot know a priori which DA method will work best. Full individual results for all methods appear in Appendix B, Tables . This represents an upper bound on baseline performance, as in practice one cannot know a priori which DA method will work best. Full individual results for all methods appear in Appendix B, Tables 10111213..

### DA techniques evaluated.

(1) TTA -Test-Time Augmentation (Wang et al. (2019); denoted TTA). Eight mirror-flipped volumes (2 3 axis-flip combinations) are inferred, logits averaged, and softmax/argmax yields the mask. This heuristic is cheap and rarely degrades performance.; denoted TTA). Eight mirror-flipped volumes (2 3 axis-flip combinations) are inferred, logits averaged, and softmax/argmax yields the mask. This heuristic is cheap and rarely degrades performance.

(2) DAE -Denoising Auto-Encoder Regularisation (Karani et al., 2021). We train a 3-layer denoising autoencoder to regularise noisy labels for both Baseline and Synth. At inference a three-layer normalisation network (3×3×3 kernels, 16 channels) is prepended to the frozen segmentor; its activation is. We train a 3-layer denoising autoencoder to regularise noisy labels for both Baseline and Synth. At inference a three-layer normalisation network (3×3×3 kernels, 16 channels) is prepended to the frozen segmentor; its activation is

$$
f (x) = exp - x 2 σ 2 , ( 1
$$

$$
)
$$

where σ is learned. For each test image the network is re-initialised and optimised for 100 steps to minimise Dice + L2 loss between segmentation logits and their DAEcleaned counterpart.

(3) TENT -Test-Time Entropy Minimisation (Wang et al., 2021). Because 3D memory limits rule out batchnorm adaptation, we instead optimise an identical normalisation network (initialised from scratch per subject) for 100 steps to minimise Shannon entropy. Because 3D memory limits rule out batchnorm adaptation, we instead optimise an identical normalisation network (initialised from scratch per subject) for 100 steps to minimise Shannon entropy

$$
H(ŷ) = - c p c (ŷ) log p c (ŷ), (2)
$$

with p c the class-c softmax probability. (4) PL Family -Pseudo-Labelling (Self-Training) (Chen et al., 2021). Three variants are tested:. Three variants are tested:

• PL: threshold softmax at τ = 1.5 N C to keep only highconfidence voxels, where N C is the total number of output classes.

• UPL: PL plus uncertainty masking via 10-sample Monte-Carlo dropout (variance > 0.05 discarded).

• DPL: full "prototype-consistency" pipeline that further removes voxels inconsistent with decoder-feature prototypes.

All PL variants fine-tune all segmentation weights for 2000 iterations with weighted cross-entropy. Common optimiser settings. Every trainable DA method (DAE, TENT, PL, UPL, DPL) uses AdamW (Loshchilov and Hutter, 2019) with learning-rate 0.002 and weight-decay 0.01. TTA is inference-only and therefore parameter-free. with learning-rate 0.002 and weight-decay 0.01. TTA is inference-only and therefore parameter-free.

### Large-Scale Pseudo-Labelling

Because the synthetic pipeline decouples image realism from label accuracy, we can tolerate imperfect pseudolabels. We therefore use Baseline + TTA to annotate 1159 chronic stroke MPRAGE scans from PLORAS Sample 1 (Seghier et al., 2016); this PLORAS-MPRAGE cohort feeds the mixed loader exactly like ATLAS.; this PLORAS-MPRAGE cohort feeds the mixed loader exactly like ATLAS.

### Experiments

### Datasets

Models were validated on four stroke-lesion datasets. We assessed the models' in-domain performance on the holdout test set for the ATLAS dataset (131 subjects, 1 mm isotropic MPRAGE). Out-of-domain (OOD) robustness was evaluated on three additional cohorts: the ISLES 2015 dataset (Maier et al., 2017) (N=28 subjects with skullstripped T1w/T2w/FLAIR/DWI), the ARC dataset (N=28 subjects with skullstripped T1w/T2w/FLAIR/DWI), the ARC dataset (Gibson et al., 2024;Johnson et al., 2024) (N=229 T2w, 202 T1w, 85 FLAIR; N=84 subjects have all three), and the hospital scans from 661 acute-stroke patients in PLORAS Sample 2 (N=106 T2w, 300 FLAIR, 255 CT), collectively referred to as PLORAS (N=229 T2w, 202 T1w, 85 FLAIR; N=84 subjects have all three), and the hospital scans from 661 acute-stroke patients in PLORAS Sample 2 (N=106 T2w, 300 FLAIR, 255 CT), collectively referred to as PLORAS (Price et al., 2010). ISLES and PLO-RAS also introduce an acute-versus-chronic shift. There is no overlap in patients between the PLORAS hospital scans (acute) and the PLORAS MPRAGE cohort described in Section 2.5.. ISLES and PLO-RAS also introduce an acute-versus-chronic shift. There is no overlap in patients between the PLORAS hospital scans (acute) and the PLORAS MPRAGE cohort described in Section 2.5.

For PLORAS, images are resampled from the original 2 mm isotropic resolution to 1 mm to maintain a single preprocessing pipeline, acknowledging that this constitutes a second resampling step.

### Experimental Design

Pre-processing and inference. All test images are reoriented to RAS, resliced to 1 mm 3 voxels, histogramnormalised and z-scored. Inference uses a 192 3 sliding window with 50 % overlap and a Gaussian blending kernel (σ = 0.125). Test-time augmentation (TTA) averages logits over all eight combinations of left-right, anteriorposterior and inferior-superior flips.

Multi-modal ensembles. When multiple MR sequences are available for the same subject (ISLES 2015, ARC) we average the per-modality logits before the softmax. This simple ensembling mimics a realistic clinical deployment.

Pseudo-label training. Pseudo-labels are generated for the PLORAS MPRAGE cohort with the Baseline + TTA model. A new Baseline and a new Synth model are then re-trained using the union of ATLAS and pseudo-labelled data, following exactly the same optimisation schedule as the originals.

### Evaluation Metrics

Prior to metric computation, predictions and ground truth are resliced to 1 mm and zero-padded to 256 3 voxels. We report Dice and Surface-Dice (Seidlitz et al., 2022) (1 mm tolerance) in the main text; HD95, absolute volume difference (AVD), absolute lesion difference (ALD), lesionwise F1, true-positive rate (TPR) and false-positive rate (FPR) appear in Appendix B. (1 mm tolerance) in the main text; HD95, absolute volume difference (AVD), absolute lesion difference (ALD), lesionwise F1, true-positive rate (TPR) and false-positive rate (FPR) appear in Appendix B.

Dice quantifies volumetric overlap (1 = perfect, 0 = none), whereas HD95 captures boundary error while being robust to outliers. AVD reports absolute volume mismatch in cm 3 . ALD counts mismatches in the number of connected components, and lesion-wise F1 scores per-lesion detection accuracy. TPR and FPR follow standard definitions.

### Comparison Methods

We evaluate four primary approaches:

(i) Baseline: A standard 3D U-Net trained solely on real MPRAGE images from the ATLAS dataset (N=419) using supervised learning. This represents the conventional approach of training on a single modality with manual annotations.

(ii) Oracle DA: The baseline model enhanced with the single best-performing domain adaptation method (selected post-hoc from TTA, TENT, DAE, PL, UPL, DPL) for each modality. This represents the theoretical upper bound of what domain adaptation can achieve when the optimal method is known -an unrealistic scenario in practice.

(iii) WMH-SynthSeg: The pre-trained white matter hyperintensity model from Laso et al. (2024); ; Fischl (2012), included as an off-the-shelf robust lesion segmentation baseline. Note that this model was trained for small periventricular WMH lesions, which differ substantially from large cortical stroke lesions., included as an off-the-shelf robust lesion segmentation baseline. Note that this model was trained for small periventricular WMH lesions, which differ substantially from large cortical stroke lesions.

(iv) Synth (Ours): Our proposed approach using synthetic data generation as described in Section 2. The model is trained on a mixture of synthetic data (generated from OASIS healthy maps + ATLAS lesion masks) and real AT-LAS MPRAGE images, following the framework illustrated in Figure 2..

For fair comparison, TTA is only applied when a model is explicitly labelled "+TTA" or when it is selected as the Oracle DA. All candidate maps are binarised via argmax on posterior probabilities without additional calibration or threshold tuning.

### Results

Overall results are shown in Figure 3, which compares (i) the Baseline, (ii) the Oracle DA (best-performing domain adaptation for Baseline only), (iii) WMH-SynthSeg, and (iv) the DA-free Synth model. Oracle DA represents the hy-pothetical best-case scenario where the optimal DA method is known a priori for each dataset/modality. Tables , which compares (i) the Baseline, (ii) the Oracle DA (best-performing domain adaptation for Baseline only), (iii) WMH-SynthSeg, and (iv) the DA-free Synth model. Oracle DA represents the hy-pothetical best-case scenario where the optimal DA method is known a priori for each dataset/modality. Tables 2, , 3, 4 and 5 provide comprehensive results for all tested domain adaptation methods on both Baseline and Synth models., 4 and 5 provide comprehensive results for all tested domain adaptation methods on both Baseline and Synth models.

### ATLAS

The ATLAS dataset represents the in-domain scenario with T1-weighted images matching our training distribution. Table 2 shows performance on the held-out test set. The conventional Baseline achieves a median Dice of 57.5 %, whereas our Synth model reaches 48.2 %, a 9.3 % gap that represents the 'price of robustness' we accept for the larger out-of-domain gains reported later. Surface Dice follows the same trend (49.4 % vs. 38.1 %).

Applying test-time augmentation (Baseline+TTA) changes Dice by <0.1 %, indicating that TTA adds little benefit when the evaluation domain is already aligned with training.

The off-the-shelf WMH-SynthSeg, trained for small periventricular WMH lesions, scores only 7.3 % Dice, confirming that cortical stroke in ATLAS lies well outside its intended scope.

A paired Wilcoxon test (Appendix Figure 7) verifies that the Baseline-Synth Dice difference is statistically significant, underscoring that domain-invariant training still sacrifices some in-domain accuracy. Voxel-level metrics in Appendix Table ) verifies that the Baseline-Synth Dice difference is statistically significant, underscoring that domain-invariant training still sacrifices some in-domain accuracy. Voxel-level metrics in Appendix Table 10 reveal the Baseline achieves a higher recall but at the cost of more false positives, suggesting a tendency to over-segment. reveal the Baseline achieves a higher recall but at the cost of more false positives, suggesting a tendency to over-segment.

Worst Scanner Syndrome. This in-domain drop is consistent with the 'Worst Scanner Syndrome' hypothesis (Moyer and Golland, 2021), which posits that many domaininvariance strategies pull feature quality toward the least informative -or noisiest -domain. As the next Results sections demonstrate, that modest in-domain penalty is offset by substantial gains on heterogeneous clinical data. The ARC dataset contains research-quality chronic stroke scans, representing a moderate domain shift from our AT-LAS training data. We expect T1w performance to be strongest given its proximity to the training domain., which posits that many domaininvariance strategies pull feature quality toward the least informative -or noisiest -domain. As the next Results sections demonstrate, that modest in-domain penalty is offset by substantial gains on heterogeneous clinical data. The ARC dataset contains research-quality chronic stroke scans, representing a moderate domain shift from our AT-LAS training data. We expect T1w performance to be strongest given its proximity to the training domain.

Table 3 shows performance on ARC, comparing Baseline, Oracle DA (best possible domain adaptation for Baseline), WMH-SynthSeg, and our Synth approach. For T1w, both baseline and Oracle DA achieve 75.2% Dice, with Synth at 72.3% -all maintaining strong performance (Appendix Figure shows performance on ARC, comparing Baseline, Oracle DA (best possible domain adaptation for Baseline), WMH-SynthSeg, and our Synth approach. For T1w, both baseline and Oracle DA achieve 75.2% Dice, with Synth at 72.3% -all maintaining strong performance (Appendix Figure 8 shows statistical significance). However, T2w reveals dramatic differences: baseline drops to 0.4% and Oracle DA to 0.1%, while Synth maintains 26.8%. FLAIR shows intermediate performance with Oracle DA at 12.4% versus Synth at 14.1%. The ensemble results are particularly striking: Oracle DA achieves only 11.7% while Synth reaches 60.2%. This pattern -strong baseline performance on T1w but catastrophic failure on other modalities even with optimal DA -demonstrates that synthetic pre-training provides fundamental robustness unachievable through post-hoc adaptation. shows statistical significance). However, T2w reveals dramatic differences: baseline drops to 0.4% and Oracle DA to 0.1%, while Synth maintains 26.8%. FLAIR shows intermediate performance with Oracle DA at 12.4% versus Synth at 14.1%. The ensemble results are particularly striking: Oracle DA achieves only 11.7% while Synth reaches 60.2%. This pattern -strong baseline performance on T1w but catastrophic failure on other modalities even with optimal DA -demonstrates that synthetic pre-training provides fundamental robustness unachievable through post-hoc adaptation.

### ISLES 2015

ISLES 2015 contains skull-stripped sub-acute stroke scans, representing our most challenging domain shift with both acquisition and pathology differences from the chronic ATLAS training data.

The results in Figure 3 and Table and Table 4 show that the Synth model outperforms the Baseline model in all modalities in regards to both Dice and Surface Dice with high statistical significance in the Dice metric evidenced in Appendix Figures 11-13. Baseline performance is near-zero across all modalities. Oracle DA shows minimal recovery: 10.5% for T1w (still below Synth's 11.0%) and 0.0% for all other modalities (T2w, FLAIR, DWI). In contrast, Synth achieves (Ours) 26.8 (23.3-30.2) 12.0 (10.4-13.6) FLAIR Baseline 12.0 (7.6-16.3) 6.3 (4.7-7.8) Oracle DA 12.4 (5.8-19.0) 7.7 (5.2-10.2) WMH-SynthSeg 2.4 (0.9-3.9) 4.3 (3.0-5.5) Synth (Ours) 14.1 (9.7-18.5) 6.6 (4.8-8.4) Ensemble Baseline 3.4 (1.3-5.4) 2.0 (0.8-3.2) Oracle DA 11.7 (8.6-14.7) 6.3 (4.7-7.8) Synth (Ours) 60.2 (56.6-63.8) 26.3 (24.3-28.4) show that the Synth model outperforms the Baseline model in all modalities in regards to both Dice and Surface Dice with high statistical significance in the Dice metric evidenced in Appendix Figures 11-13. Baseline performance is near-zero across all modalities. Oracle DA shows minimal recovery: 10.5% for T1w (still below Synth's 11.0%) and 0.0% for all other modalities (T2w, FLAIR, DWI). In contrast, Synth achieves (Ours) 26.8 (23.3-30.2) 12.0 (10.4-13.6) FLAIR Baseline 12.0 (7.6-16.3) 6.3 (4.7-7.8) Oracle DA 12.4 (5.8-19.0) 7.7 (5.2-10.2) WMH-SynthSeg 2.4 (0.9-3.9) 4.3 (3.0-5.5) Synth (Ours) 14.1 (9.7-18.5) 6.6 (4.8-8.4) Ensemble Baseline 3.4 (1.3-5.4) 2.0 (0.8-3.2) Oracle DA 11.7 (8.6-14.7) 6.3 (4.7-7.8) Synth (Ours) 60.2 (56.6-63.8) 26.3 (24.3-28.4)

11.0% (T1w), 11.1% (T2w), 21.2% (FLAIR), and 5.6% (DWI). The ensemble demonstrates the starkest contrast: 0.0% for both baseline and Oracle DA versus 42.3% for Synth.

The poor performance correlates with high false positive rates rather than missed detections (Appendix Table 12), suggesting models struggle with tissue discrimination in this domain. When baseline predictions fail catastrophically, no DA method can recover meaningful performance.), suggesting models struggle with tissue discrimination in this domain. When baseline predictions fail catastrophically, no DA method can recover meaningful performance.

It is also evident from Table 4 that the model performance is highly dependent on the choice of image sequence available. The ensemble improves performance over individual sequences in ISLES2015 (Table that the model performance is highly dependent on the choice of image sequence available. The ensemble improves performance over individual sequences in ISLES2015 (Table 4) but shows mixed results in ARC, suggesting dataset-specific benefits rather than universal improvement. Although we only show the upper limit of an ensemble of all four sequences, it is expected that in cases where fewer sequences are available we will still observe constructive gains from post-hoc ensembling.) but shows mixed results in ARC, suggesting dataset-specific benefits rather than universal improvement. Although we only show the upper limit of an ensemble of all four sequences, it is expected that in cases where fewer sequences are available we will still observe constructive gains from post-hoc ensembling.

Table 4: Median results on the ISLES2015 dataset (N=28). Best score shown in bold. Student's t distribution 95% confidence intervals given in brackets. 'Oracle DA' represents the hypothetical best-case scenario where optimal DA method is known a priori for each dataset/modality and applied to the baseline model. Results in Table 5 demonstrate the most extreme performance gap between conventional training and our synthetic approach. On this challenging clinical dataset, Oracle DA achieves at most 0.2% Dice (CT modality), while our Synth model achieves 11.9%, 25.4%, and 11.3% for T2w, FLAIR, and CT respectively. The near-total failure of both baseline and Oracle DA on real clinical data -where acquisition protocols, slice thickness, and image quality vary substantially -validates our core hypothesis: domain adaptation cannot substitute for domain-invariant training when deployment conditions diverge significantly from training data. Even WMH-SynthSeg, despite being trained for robustness, achieves only 0.0-4.9% Dice, likely due to its focus on small periventricular lesions rather than large cortical strokes. Full results for all individual DA methods are provided in Appendix Table demonstrate the most extreme performance gap between conventional training and our synthetic approach. On this challenging clinical dataset, Oracle DA achieves at most 0.2% Dice (CT modality), while our Synth model achieves 11.9%, 25.4%, and 11.3% for T2w, FLAIR, and CT respectively. The near-total failure of both baseline and Oracle DA on real clinical data -where acquisition protocols, slice thickness, and image quality vary substantially -validates our core hypothesis: domain adaptation cannot substitute for domain-invariant training when deployment conditions diverge significantly from training data. Even WMH-SynthSeg, despite being trained for robustness, achieves only 0.0-4.9% Dice, likely due to its focus on small periventricular lesions rather than large cortical strokes. Full results for all individual DA methods are provided in Appendix Table 13. A number of samples are also visualised for this dataset in Figure . A number of samples are also visualised for this dataset in Figure 4..

### Table 5: Median results on the PLORAS dataset (N=661).

Best score shown in bold. Student's t distribution 95% confidence intervals given in brackets. 'Oracle DA' represents the hypothetical best-case scenario where optimal DA method is known a priori for each dataset/modality and applied to the baseline model.

Modality Model Dice (%) Surface Dice (%) T2w Baseline 0.0 (0.0-1.3) 0.1 (0.0-0.6) Oracle DA 0.1 (0.0-2.2) 0.1 (0.0-0.8) WMH-SynthSeg 4.9 (4.0-5.8) 0.0 (0.0-0.0) Synth (Ours) 11.9 (6.7-17.1) 8.4 (6.1-10.7) FLAIR Baseline 0.0 (0.0-0.4) 0.0 (0.0-0.2) Oracle DA 0.0 (0.0-0.4) 0.0 (0.0-0.2) WMH-SynthSeg 4.6 (4.2-5.0) 0.0 (0.0-0.0) Synth (Ours) 25.4 (22.5-28.3) 8.5 (7.4-9.6) CT Baseline 0.0 (0.0-1.1) 0.0 (0.0-0.5) Oracle DA 0.2 (0.0-0.5) 0.3 (0.0-0.7) WMH-SynthSeg 0.0 (0.0-0.0) 0.0 (0.0-0.0) Synth (Ours) 11.3 (8.0-14.6) 7.9 (6.0-9.8)

### Domain Adaptation

Tables 3, 4 and 5 demonstrate that even with oracle selection of the optimal domain adaptation method, the baseline model cannot match Synth performance in out-of-domain settings. We evaluated six established DA techniques (TTA, TENT, DAE, PL, UPL, DPL) on both baseline and Synth models to determine whether domain adaptation could enable baseline generalisation. Complete results in Appendix Tables 11-13 consistently show that poor initial baseline predictions prevent effective adaptation. In contrast, when applied to our Synth model, several DA methods yield substantial improvements, demonstrating the potential for compound gains when robust pre-training is combined with appropriate post-hoc adaptation. Analysis of complete DA results (Appendix Tables 111213) reveals DAE as the most consistently effective method for Synth. DAE achieves substantial improvements: ARC T2w increases from 26.8% to 54.3% Dice, ISLES T1w from 11.0% to 31.1%, and PLORAS CT from 11.3% to 23.9%. TTA provides modest gains for PLORAS FLAIR (25.4% to 29.4%), while TENT benefits ARC FLAIR (14.1% to 36.1%). Pseudo-labeling methods (PL, UPL, DPL) show high variance -occasionally strong but often catastrophic.) reveals DAE as the most consistently effective method for Synth. DAE achieves substantial improvements: ARC T2w increases from 26.8% to 54.3% Dice, ISLES T1w from 11.0% to 31.1%, and PLORAS CT from 11.3% to 23.9%. TTA provides modest gains for PLORAS FLAIR (25.4% to 29.4%), while TENT benefits ARC FLAIR (14.1% to 36.1%). Pseudo-labeling methods (PL, UPL, DPL) show high variance -occasionally strong but often catastrophic.

The key finding: while DA cannot rescue baseline models trained on narrow data, it can enhance robustly pretrained models. Synth+DAE consistently outperforms both Synth alone and Baseline+Oracle DA, demonstrating complementary benefits.

Our domain adaptation experiments serve primarily to establish an upper bound on baseline model performance. Even with oracle knowledge of the optimal DA method for each dataset/modality combination -an unrealistic scenario in practice -the baseline model cannot match Synth performance in out-of-domain settings. Full DA results for all methods appear in Appendix Tables 111213. While individual DA methods show varied effectiveness, the key finding is that synthetic pre-training provides robustness that cannot be recovered through test-time adaptation alone. This underscores the value of appearance-invariant training, even with our current limitations in modelling stroke heterogeneity through single Gaussian distributions. Domain adaptation results varied substantially by method and modality, with no single approach providing consistent improvements. We present all tested combinations rather than cherry-picking optimal results, as real-world deployment would lack oracle knowledge of the best DA method for unseen data.. While individual DA methods show varied effectiveness, the key finding is that synthetic pre-training provides robustness that cannot be recovered through test-time adaptation alone. This underscores the value of appearance-invariant training, even with our current limitations in modelling stroke heterogeneity through single Gaussian distributions. Domain adaptation results varied substantially by method and modality, with no single approach providing consistent improvements. We present all tested combinations rather than cherry-picking optimal results, as real-world deployment would lack oracle knowledge of the best DA method for unseen data.

### Ablation: To mix or not to mix?

For all experiments shown thus far, the Synth model used a mix of both synthetic data and the real ATLAS dataset. In order to evaluate whether this decision is justified, an ablation is performed where the Synth model using mixed real/synthetic data is compared to a model trained with only synthetic data. This model is trained in the exact same manner as described previously for the baseline and Synth models. Results for the test datasets ATLAS, ARC and ISLES 2015 are shown in Tables 6, 7 and 8 respectively.

The results reveal a nuanced picture. For in-domain and near-domain T1w data (ATLAS, ARC), mixing with real data provides clear benefits, with pure synthetic training achieving only 19.7% and 46.7% Dice respectively compared to 48.2% and 72.3% for mixed training. However, for several out-of-domain scenarios, pure synthetic training surprisingly often outperforms mixed training: ARC T2w (62.6% vs 26.8%), ISLES T1w (30.4% vs 11.0%), and ISLES FLAIR (37.2% vs 21.2%). This suggests that including real T1w data may inadvertently bias the model toward T1w-specific features, reducing generalisation to other modalities. The T2w and FLAIR improvements with pure synthetic training may reflect these sequences' different tissue contrast patterns, which are better captured by unbiased synthetic variation than by a model partially trained on T1w-specific features. Further improving the realism of the generated lesions with methods proposed in Liu et al. (2025) may aid in closing the gap for the model trained only on synthetic data, potentially achieving both in-domain performance and out-of-domain generalisation. may aid in closing the gap for the model trained only on synthetic data, potentially achieving both in-domain performance and out-of-domain generalisation.

### Use of Pseudo-labelling

The use of synthetic data for semi-supervised pseudo-label training is also explored through the use of the PLORAS MPRAGE dataset (N=1159), distinct from the PLORAS 6 and Tables 6, 7 and 8 demonstrate an overall positive effect of pseudo labels in the Synth model, with the exception of FLAIR images in ISLES 2015. and Tables 6, 7 and 8 demonstrate an overall positive effect of pseudo labels in the Synth model, with the exception of FLAIR images in ISLES 2015.

Notable improvements include: ARC T2w increasing from 26.8% to 37.2% Dice, ISLES T1w from 11.0% to 17.9%, and ARC ensemble from 60.2% to 69.4%. The baseline model shows minimal improvement with pseudolabels in out-of-domain scenarios (e.g., ISLES ensemble remains at 0.0%), suggesting that pseudo-labelling cannot overcome fundamental domain shift. The FLAIR degradation in ISLES 2015 (21.2% to 4.2%) may indicate that pseudo-labels from chronic MPRAGE data introduce biases incompatible with acute FLAIR lesion appearance. The in-domain ATLAS dataset also shows a notable improvement for both models as a result of pseudo labels, with a more marked increase in the baseline model, suggesting Table 8: Median results for ablation and pseudo-label experiments on the ISLES 2015 test set (N=28). Best score shown in bold. Student's t distribution 95% confidence intervals given in brackets. Modality Model Dice (%) Surface Dice (%) T1w Baseline 0.0 (0.0-7.4) 0.0 (0.0-4.8) Baseline+Pseudo 0.3 (0.0-9.1) 0.0 (0.0-4.0) Synth (no real data) 30.4 (21.3-39.6) 9.4 (4.1-14.8) Synth (Ours) 11.0 (1.2-20.8) 3.4 (0.0-8.5) Synth+Pseudo 17.9 (7.6-28.3) 8.0 (2.1-13.8) T2w Baseline 0.0 (0.0-0.5) 0.3 (0.0-1.0) Baseline+Pseudo 0.1 (0.0-0.7) 0.4 (0.0-1.1) Synth (no real data) 7.4 (0.0-18.4) 8.1 (3.0-13.1) Synth (Ours) 11.1 (0.7-21.6) 7.1 (2.7-11.5) Synth+Pseudo 18.1 (7.1-29.1) 9.5 (4.4-14.5) FLAIR Baseline 0.0 (0.0-0.0) 0.0 (0.0-0.0) Baseline+Pseudo 0.0 (0.0-0.3) 0.0 (0.0-0.4) Synth (no real data) 37.2 (25.5-48.9) 16.0 (10.8-21.1) Synth (Ours) 21.2 (8.5-34.0) 14.7 (9.0-20.3) Synth+Pseudo 4.2 (0.0-17.3) 3.9 (0.0-9.8) DWI Baseline 0.0 (0.0-0.0) 0.0 (0.0-0.0) Baseline+Pseudo 0.0 (0.0-0.4) 0.0 (0.0-0.0) Synth (no real data) 8.2 (0.0-16.8) 5.1 (2.0-8.3) Synth (Ours) 5.6 (0.0-14.4) 4.1 (0.8-7.4) Synth+Pseudo 5.6 (0.0-14.7) 4.3 (0.8-7.8) Ensemble Baseline 0.0 (0.0-0.0) 0.0 (0.0-0.0) Baseline+Pseudo 0.0 (0.0-0.0) 0.0 (0.0-0.0) Synth (no real data) 27.2 (15.7-38.8) 15.6 (10.3-20.9) Synth (Ours) 42.3 (30.2-54.5) 19.1 (12.7-25.4) Synth+Pseudo 50.1 (37.6-62.6) 24.9 (18.7-31.1)

pseudo-labels effectively expand the training distribution when the domain gap is small. These results across four diverse datasets -from researchquality ATLAS to challenging clinical PLORAS scans -re-veal consistent patterns that warrant deeper analysis of the fundamental principles underlying domain robustness.

### Discussion

Our results reveal fundamental principles about domain robustness in medical image segmentation. The consistent failure of domain adaptation to rescue baseline models -even with oracle selection of the optimal method -demonstrates that post-hoc adaptation cannot substitute for domaininvariant training. This finding challenges prevailing assumptions about the sufficiency of test-time adaptation for clinical deployment.

The compound effect observed with Synth+DAE suggests that synthetic pre-training and test-time adaptation address different aspects of domain shift. Synthetic training provides feature representations invariant to appearance changes, while DA methods fine-tune these representations to specific test-time distributions. However, this synergy only emerges when the base model already possesses sufficient robustness -when initial predictions fail catastrophically (as with baseline models on ISLES 2015 and PLORAS), no amount of adaptation can recover performance.

The variable benefit of multi-modal ensembling -substantial for ISLES 2015 but mixed for ARC -indicates that fusion strategies must consider dataset-specific characteristics rather than assuming universal improvement. Clinical deployment should balance single-modality robustness with opportunistic multi-modal fusion.

### Limitations and Future Directions

Appendix A confirms that 20-40% of lesions in several cohorts are multimodal, underscoring the central limitation of our single-Gaussian sampling. While our spatially-varying approach introduces intra-lesion heterogeneity, it does not address the multi-modal nature of stroke appearance. Future implementations should explore mixture models to capture lesions that simultaneously exhibit hyper-and hypointense regions within the same pathology. It may also be advantageous to paste several lesions per subject, each assigned its own independently sampled intensity profile. This would allow a single synthetic brain to display chronic hypointense scars alongside acute hyperintense infarcts, better reflecting the heterogeneous lesion chronology typically observed in stroke cohorts.

Additional qualitative results in Appendix Figures 18 -23 display further positive and negative results across the available image modalities within the PLORAS dataset. A number of failure modes may be observed here, such as missing cerebellar lesions or under-segmenting large hemispheric lesions.

Although results in the paper provide strong evidence of the raw value of the segmentation metrics, it is important to further validate that the predictions made from the trained model are useful for downstream tasks. This could be In the context of neuroimaging data, there are many shifts in domain beyond those related to scanner sequence and resolution targeted in this work. Shifts related to anatomical shape, brought on by changes in demographics or the presence of confounding conditions such as atrophy are of equal importance to shifts in image appearance. Prior work has attempted to model such factors through the use of causal inference and counterfactual generation (Pawlowski et al., 2020;Pombo et al., 2023;Ribeiro et al., 2023). It is conceivable that such an approach could be used to introduce morphological changes to the healthy tissue maps to create a model that is agnostic to shifts in domains related to both shape and appearance.. It is conceivable that such an approach could be used to introduce morphological changes to the healthy tissue maps to create a model that is agnostic to shifts in domains related to both shape and appearance.

While this study focused on stroke, the results are expected to translate to other similar domains, such as haemorrhage and glioblastoma. Future work will compare the impact of the mixing proportions of real and synthetic data. Additionally, the utility of multi-modal data in a multichannel model will be examined versus post-hoc averaging of individual modality predictions. Lesions often appear differently across modalities, so a model trained with multichannel inputs is expected to leverage these differences more effectively. This relationship between multi-channel inputs may better be modelled using quantitative MRI data.

### Conclusion

In this study, we introduced a novel synthetic data generation and training framework for stroke lesion segmentation, building on the success of prior works in healthy brain parcellation. A model trained using this novel framework was evaluated on a wide range of datasets covering research and clinical data, and chronic and acute stroke pathologies. Our experiments demonstrate that synthetic pre-training provides fundamental robustness unachievable through testtime adaptation alone. While accepting a 9.3% median Dice reduction in-domain (48.2% vs 57.5%), our approach maintains performance where conventional methods fail entirely. Even with oracle knowledge of the optimal domain adaptation method -an unrealistic scenario in practiceconventionally-trained models cannot match our synthetic approach in out-of-domain settings.

Our results demonstrate that even imperfect appearance modelling can provide substantial benefits for cross-modality segmentation. Incorporating recent contemporary work on realistic lesion simulation (Liu et al., 2025) may further improve performance both in-and out-of-distribution. may further improve performance both in-and out-of-distribution.

Qualitative results in Figure 4 demonstrate the potential of the model as a starting checkpoint in an active-learning framework such as MONAI Label demonstrate the potential of the model as a starting checkpoint in an active-learning framework such as MONAI Label (Diaz-Pinto et al., 2024), where predictions may be refined to generate training data for task-specific fine-tuning. The uncertainty of healthytissue segmentations via MC dropout may also provide an effective heuristic for prioritising items to refine/label in such a framework , where predictions may be refined to generate training data for task-specific fine-tuning. The uncertainty of healthytissue segmentations via MC dropout may also provide an effective heuristic for prioritising items to refine/label in such a framework (Nath et al., 2021)..

Results also showed the potential benefit of the model for semi-supervised learning through pseudo labelling, where predicted labels may be fed back into the model with much less caution than is typically required when training with real images (Pham et al., 2021;Xu et al., 2024). The ablation study revealed that pure synthetic training can outperform mixed training for certain out-of-domain modalities, suggesting opportunities for further optimisation of the real/synthetic data balance. A simple solution to further reinforce this may be to train a real/fake discriminator to perform automated quality-control and threshold optimisation for the generated binary maps.. The ablation study revealed that pure synthetic training can outperform mixed training for certain out-of-domain modalities, suggesting opportunities for further optimisation of the real/synthetic data balance. A simple solution to further reinforce this may be to train a real/fake discriminator to perform automated quality-control and threshold optimisation for the generated binary maps.

Our approach reduces reliance on domain-specific training data and helps bridge the gap between research-grade and clinical scans to improve clinical stroke neuroimaging workflows, providing a foundation for more robust and widely applicable lesion segmentation tools.

Table 11: Mean results on the ARC dataset (N=229). Best score per column is shown in bold. FPR values are shown as a multiple of 1000.

> (a) FreeSurfer (Puonti et al., 2016) (b) MultiBrain (Brudfors et al., 2020

> 1 Figure 1 : Figure 1: Sample generated images using different labels for a single subject. 1a: FreeSurfer anatomical labels. 1b: MultiBrain tissue labels. 1c: MultiBrain tissue labels masked to simulate skull-stripping.

> Output channels. Background + Gray Matter (GM) + White Matter (WM) + GM/WM Partial Volume + Cerebrospinal Fluid (CSF) + Lesion (total six channels) for Synth; Background + Lesion for Baseline. When real images (binary GT) enter the mixed loader we mask the loss to the lesion channel only. Training schedule. 192 3 crops, batch 1, 1 200 epochs × 500 iterations (=6 × 10 5 updates), AdamW (η 0 = 10 -4 , weight-decay 0.01, poly LR decay with power 0.9), combined Dice + CE loss, dropout 0.2, gradient-norm clip 12. Data-loader mixing. Synthetic : Real ratio = 2 579 : 419 (mirrors sample counts). Real MPRAGE images receive the same spatial/intensity transforms as synthetic batches.

> 3 Figure 3 : Figure3: Dice and Surface Dice metrics for all reported datasets. 'Oracle DA' represents the hypothetical best-case scenario where optimal DA method is known a priori for each dataset/modality and applied to the baseline model.

> represents the most extreme domain shift with real clinical data exhibiting large diversity in acquisition protocols and slice thickness. We expect minimal baseline performance given these challenging conditions. For all available modalities, the Synth model outperforms the baseline with statistical significance in Dice (see Appendix Figures15 -17).

> 4 Figure 4 : Figure 4: Sample visualisations in the PLORAS dataset. Green indicates a true positive prediction, red a false positive, and blue a false negative.

> 5 Figure 5 : Figure5: Dice and Surface Dice metrics for all reported datasets, for models trained on different combinations of real/synthetic data.

> 6 Figure 6 : Figure6: Dice and Surface Dice metrics for all reported datasets, for models trained with/without an additional training dataset of MPRAGE images and pseudo-labels.

> validated by comparing predictivity of the segmentation masks for functional scores such as the Comprehensive Aphasia Test (CAT) (Hope et al., 2024) or the NIH Stroke Score (NIHSS).

> 7 Figure 7 : Figure 7: Wilcoxon signed-rank test values for Dice metric measurements in the ATLAS T1w dataset.

> 8 Figure 8 : Figure 8: Wilcoxon signed-rank test values for Dice metric measurements in the ARC T1w dataset.

> 9 Figure 9 : Figure 9: Wilcoxon signed-rank test values for Dice metric measurements in the ARC T2w dataset.

> 10 Figure 10 : Figure 10: Wilcoxon signed-rank test values for Dice metric measurements in the ARC FLAIR dataset.

> 11 Figure 11 : Figure 11: Wilcoxon signed-rank test values for Dice metric measurements in the ISLES 2015 T1w dataset.

> 12 Figure 12 : Figure 12: Wilcoxon signed-rank test values for Dice metric measurements in the ISLES 2015 T2w dataset.

> 13 Figure 13 : Figure 13: Wilcoxon signed-rank test values for Dice metric measurements in the ISLES 2015 FLAIR dataset.

> 14 Figure 14 : Figure 14: Wilcoxon signed-rank test values for Dice metric measurements in the ISLES 2015 DWI dataset.

> 15 Figure 15 : Figure 15: Wilcoxon signed-rank test values for Dice metric measurements in the PLORAS T2w dataset.

> 16 Figure 16 : Figure 16: Wilcoxon signed-rank test values for Dice metric measurements in the PLORAS FLAIR dataset.

> 18 Figure 18 : Figure 18: Sample visualisations of successful cases in the PLORAS T2w dataset. Green indicates a true positive prediction, red a false positive, and blue a false negative.

> 19 Figure 19 : Figure 19: Sample visualisations of failure cases in the PLORAS T2w dataset. Green indicates a true positive prediction, red a false positive, and blue a false negative.

> 20 Figure 20 : Figure 20: Sample visualisations of successful cases in the PLORAS FLAIR dataset. Green indicates a true positive prediction, red a false positive, and blue a false negative.

> 21 Figure 21 : Figure 21: Sample visualisations of failure cases in the PLORAS FLAIR dataset. Green indicates a true positive prediction, red a false positive, and blue a false negative.

> 22 Figure 22 : Figure 22: Sample visualisations of successful cases in the PLORAS CT dataset. Green indicates a true positive prediction, red a false positive, and blue a false negative.

> 23 Figure 23 : Figure 23: Sample visualisations of failure cases in the PLORAS CT dataset. Green indicates a true positive prediction, red a false positive, and blue a false negative.

> 1 Table 1 : Parameter ranges for every image-quality augmentation used during synthetic training. † indicates augmentations applied only to synthetic data and not to real ATLAS samples.

> 2 Table 2 : Median results on the ATLAS hold-out set (N=131). Best score shown in bold. Student's t distribution 95% confidence intervals given in brackets.

> 3 Table 3 : Median results on the ARC dataset (N=229).Best score shown in bold. Student's t distribution 95% confidence intervals given in brackets. 'Oracle DA' represents the hypothetical best-case scenario where optimal DA method is known a priori for each dataset/modality and applied to the baseline model.

> 6 Table 6 : Median results for ablation and pseudo-label experiments on the ATLAS test set (N=131). Best score shown in bold. Student's t distribution 95% confidence intervals given in brackets.

> 7 Table 7 : Median results for ablation and pseudo-label experiments on the ARC test set (N=229). Best score shown in bold. Student's t distribution 95% confidence intervals given in brackets.

## References

- John Ashburner, Karl J Friston. Unified segmentation. NeuroImage. 2005
- Suyash Mohan, and others. The RSNA-ASNR-MICCAI BraTS 2021 Benchmark on Brain Tumor Segmentation and Radiogenomic Classification. 2021
- Lesionmix: A lesion-level data augmentation method for medical image segmentation. 2023
- Benjamin Billot, Stefano Cerri, Koen Van Leemput, Adrian V Dalca, Juan Eugenio Iglesias. Joint segmentation of multiple sclerosis lesions and brain anatomy in MRI scans of any contrast and resolution with CNNs. 2021 IEEE 18th International Symposium on Biomedical Imaging (ISBI). 2021
- Benjamin Billot, Douglas N Greve, Oula Puonti, Axel Thielscher, Koen Van Leemput. SynthSeg: Segmentation of brain MRI scans of any contrast and resolution without retraining. Medical Image Analysis. 2023
- Flexible Bayesian Modelling for Nonlinear Image Registration. 2020
- Monai: An open-source framework for deep learning in healthcare. 2022
- Stefano Cerri, Oula Puonti, Dominik S Meier, Jens Wuerfel, Mark Mühlau. A contrast-adaptive method for simultaneous whole-brain and lesion segmentation in multiple sclerosis. NeuroImage. 2021
- Source-free domain adaptive fundus image segmentation with denoised pseudo-labeling. 2021
- Neurovascular segmentation in soct with deep learning and synthetic training data. 2024
- Soft-CP: A credible and effective data augmentation for semantic segmentation of medical lesions. 2022
- Adrian V Dalca, John Guttag, Mert R Sabuncu. Anatomical priors in convolutional networks for unsupervised biomedical segmentation. 2018 IEEE/CVF Conference on Computer Vision and Pattern Recognition. 2018
- A robust ensemble algorithm for ischemic stroke lesion segmentation: Generalizability and clinical utility beyond the isles challenge. 2024
- Andres Diaz-Pinto, Sachidanand Alle, Vishwesh Nath, Yucheng Tang, Alvin Ihsani. [Monai label: A framework for ai-assisted interactive labeling of 3d medical images](https://doi.org/10.1016/j.media.2024.103207). Medical Image Analysis. 2024
- Bruce Fischl, Freesurfer. NeuroImage. 2012
- Makayla Gibson, Roger Newman-Norlund, Leonardo Bonilha, Julius Fridriksson, Gregory Hickok. [The Aphasia Recovery Cohort, an open-source chronic stroke repository](https://doi.org/10.1038/s41597-024-03819-7). Scientific Data. 2024
- Cortical analysis of heterogeneous clinical brain mri scans for large-scale neuroimaging studies. 2023
- Recon-all-clinical: Cortical surface reconstruction and analysis of heterogeneous clinical brain mri. 2024
- Karthik Gopinath, Andrew Hoopes, Daniel C Alexander, Steven E Arnold, Yael Balbastre. [Synthetic data in generalizable, learning-based neuroimaging](https://doi.org/10.1162/imag_a_00337). Imaging Neuroscience. 2024
- J A Hartigan, P M Hartigan. [The dip test of unimodality](https://doi.org/10.1214/aos/1176346577). The Annals of Statistics. 1985
- Malte Hoffmann, Benjamin Billot, Juan Eugenio Douglas N Greve, Bruce Iglesias, Fischl. Synthmorph: learning contrast-invariant registration without acquired images. IEEE Transactions on Medical Imaging. 2022
- Malte Hoffmann, Andrew Hoopes, Bruce Fischl, Adrian V Dalca. Anatomy-specific acquisition-agnostic affine registration learned from fictitious images. Medical Imaging 2023: Image Processing. 2023
- Malte Hoffmann, Andrew Hoopes, Bruce Douglas N Greve, Adrian V Fischl, Dalca. Anatomy-aware and acquisition-agnostic joint registration with synthmorph. Imaging Neuroscience. 2024
- Continuous lesion images drive more accurate predictions of outcomes after stroke than binary lesion images. 2024
- Juan E Iglesias, Benjamin Billot, Yaël Balbastre, Colin Magdamo, Steven E Arnold. [Synthsr: A public ai tool to turn heterogeneous clinical brain scans into high-resolution t1-weighted images for 3d morphometry](https://doi.org/10.1126/sciadv.add3607). Science Advances. 2023
- Juan Eugenio, Iglesias, Benjamin Billot, Yaël Balbastre, Azadeh Tabari, John Conklin. Joint super-resolution and synthesis of 1 mm isotropic MP-RAGE volumes from clinical MRI exams with scans of different orientation, resolution and contrast. NeuroImage. 2021
- Fabian Isensee, Paul F Jaeger, A A Simon, Jens Kohl, Klaus H Petersen, Maier-Hein. Net: a selfconfiguring method for deep learning-based biomedical image segmentation. Nature Methods. 2020
- nnu-net revisited: A call for rigorous validation in 3d medical image segmentation. 2024
- Lisa Johnson, Roger Newman-Norlund, Alex Teghipco. Progressive lesion necrosis is related to increasing aphasia severity in chronic stroke. NeuroImage: Clinical. 2024
- Neerav Karani, Ertunc Erdil, Krishna Chaitanya, Ender Konukoglu. [Test-time adaptable neural networks for robust medical image segmentation](https://doi.org/10.1016/j.media.2020.101907). Medical Image Analysis. 2021
- Quantifying white matter hyperintensity and brain volumes in heterogeneous clinical and low-field portable mri. 2024
- Hans Lassmann. Multiple sclerosis pathology. Cold Spring Harbor Perspectives in Medicine. 2018
- Qingyun Li, Zhibin Yu, Yubo Wang, Haiyong Zheng. [Tumorgan: A multi-modal data augmentation framework for brain tumor segmentation](https://doi.org/10.3390/s20154203). Sensors. 2020
- PEPSI: Pathology-Enhanced Pulse-Sequence-Invariant Representations for Brain MRI. 2024
- Unraveling normal anatomy via fluid-driven anomaly randomization. 2025
- Decoupled weight decay regularization. 2019
- Robert Loughnan, Diego L Lorca-Puls, Andrea Gajardo-Vidal, Valeria Espejo-Videla, Céline R Gillebert. [Generalizing post-stroke prognoses from research data to clinical data](https://doi.org/10.1016/j.nicl.2019.102005). NeuroImage: Clinical. 2019
- Oskar Maier, Bjoern H Menze, Janina Von Der, Levin Gablentz, Mattias P Häni, Heinrich. ISLES 2015 -a public evaluation benchmark for ischemic stroke lesion segmentation from multispectral MRI. Medical Image Analysis. 2017
- Local gamma augmentation for ischemic stroke lesion segmentation on mri. 2024
- Harmonization and the worst scanner syndrome. 2021
- Vishwesh Nath, Dong Yang, Bennett A Landman, Daguang Xu, Holger R Roth. [Diminishing uncertainty within the training pool: Active learning for medical image segmentation](https://doi.org/10.1109/TMI.2020.3048055). IEEE Transactions on Medical Imaging. 2021
- Mohamad Thanh N Nguyen, Urs Abdalkader, Zhongming Fischer, Simon Qiu, Nagel. [Endovascular management of acute stroke](https://doi.org/10.1016/S0140-6736(24)01410-7). The Lancet. 2024
- Deep structural causal models for tractable counterfactual inference. 2020
- Meta pseudo labels. 2021
- Guilherme Pombo, Robert Gray, Jorge Cardoso, Sebastien Ourselin, Geraint Rees, John Ashburner, Parashkev Nachev. Equitable modelling of brain imaging by counterfactual augmentation with morphologically constrained 3D deep generative models. Med. Image Anal. 2023
- Cathy J Price, Mohamed L Seghier, Alex P Leff. Predicting language outcome and recovery after stroke: The PLORAS system. Nature Reviews Neurology. 2010
- Oula Puonti, Juan Eugenio Iglesias, Koen Van Leemput. [Fast and sequence-adaptive whole-brain segmentation using parametric Bayesian modeling](https://doi.org/10.1016/j.neuroimage.2016.09.011). NeuroImage. 2016
- High fidelity image counterfactuals with probabilistic causal models. 2023
- Mohamed L Seghier, Anil Ramlackhansingh, Jenny Crinion, Alexander P Leff, Cathy J Price. [Lesion identification using unified segmentation-normalisation models and fuzzy clustering](https://doi.org/10.1016/j.neuroimage.2008.03.028). NeuroImage. 2008
- Mohamed L Seghier, Elnas Patel, Susan Prejawa, Sue Ramsden, Andre Selmer. [The PLORAS Database: A data repository for Predicting Language Outcome and Recovery After Stroke](https://doi.org/10.1016/j.neuroimage.2015.03.083). NeuroImage. 2016
- Silvia Seidlitz, Jan Sellner, Jan Odenthal, Berkin Özdemir, Alexander Studier-Fischer. [Robust deep learningbased semantic organ segmentation in hyperspectral images](https://doi.org/10.1016/j.media.2022.102488). Medical Image Analysis. 2022
- Tent: Fully test-time adaptation by entropy minimization. 2021
- Automatic Brain Tumor Segmentation Using Convolutional Neural Networks with Test-Time Augmentation. 2019
- Moucheng Xu, Yukun Zhou, Chen Jin, Marius De Groot, Daniel C Alexander. [Expectation maximisation pseudo labels](https://doi.org/10.1016/j.media.2024.103125). Medical Image Analysis. 2024
- Xinru Zhang, Chenghao Liu, Ni Ou, Xiangzhu Zeng, Xiaoliang Xiong. [CarveMix: A Simple Data Augmentation Method for Brain Lesion Segmentation](https://doi.org/10.1007/978-3-030-87193-2_19). 2021. ISBN 9783030871932
