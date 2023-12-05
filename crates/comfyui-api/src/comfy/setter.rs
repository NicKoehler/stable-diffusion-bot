use anyhow::{anyhow, Context};

use crate::{
    comfy::getter::{find_node, guess_node, GetExt},
    models::*,
};

/// Extension methods for `Prompt` to use Setters to set values on nodes.
pub trait SetterExt<T, N>
where
    N: Node + 'static,
{
    /// Uses a heuristic to find a `Node` and set the value on it.
    ///
    /// # Inputs
    ///
    /// * `value` - The value to set.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set<S>(&mut self, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>;

    /// Finds a `Node` leading into the given `output_node` and sets the value on it.
    ///
    /// # Inputs
    ///
    /// * `output_node` - The id of the node to path from.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_from<S>(&mut self, output_node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>;

    /// Sets the value on the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `node` - The id of the node to set the value on.
    /// * `value` - The value to set.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_node<S>(&mut self, node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>;
}

/// Extension methods for `Prompt` to set values on nodes.
pub trait SetExt<N>
where
    N: Node + 'static,
{
    /// Uses a heuristic to find a `Node` and set the value on it.
    ///
    /// # Inputs
    ///
    /// * `f` - A function that takes a mutable reference to the node and returns a `Result`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_with<F>(&mut self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>;

    /// Sets the value on the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `node` - The id of the node to set the value on.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_node_with<F>(&mut self, node: &str, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>;
}

impl<T, N: Node + 'static> SetterExt<T, N> for Prompt {
    fn set<S>(&mut self, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>,
    {
        S::from(value).set(self)
    }

    fn set_from<S>(&mut self, output_node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>,
    {
        S::from(value).set_from(self, output_node)
    }

    fn set_node<S>(&mut self, node: &str, value: T) -> anyhow::Result<()>
    where
        S: Setter<T, N>,
    {
        S::from(value).set_node(self, node)
    }
}

impl<N: Node + 'static> SetExt<N> for Prompt {
    fn set_with<F>(&mut self, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>,
    {
        if let Some(node) = guess_node::<N>(self, None) {
            f(as_node_mut::<N>(node).context("Failed to cast node")?)
        } else {
            Err(anyhow!("Failed to find node"))
        }
    }

    fn set_node_with<F>(&mut self, node: &str, f: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut N) -> anyhow::Result<()>,
    {
        f(self.get_node_mut(node)?)
    }
}

/// A trait for setting values on nodes.
pub trait Setter<T, N>
where
    N: Node + 'static,
    Self: From<T>,
{
    /// Uses a heuristic to find a `Node` and set the value on it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set(&self, prompt: &mut Prompt) -> anyhow::Result<()> {
        let node = if let Some(node) = guess_node::<N>(prompt, None) {
            node
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.set_value(node)
    }

    /// Finds a `Node` leading into the given `output_node` and sets the value on it.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_from(&self, prompt: &mut Prompt, output_node: &str) -> anyhow::Result<()> {
        let node = if let Some(node) = Self::find_node(prompt, Some(output_node)) {
            prompt
                .get_node_by_id_mut(&node)
                .context("Failed to find node")?
        } else {
            return Err(anyhow!("Failed to find node"));
        };
        self.set_value(node)
    }

    /// Sets the value on the node with id `node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    /// * `node` - The id of the node to set the value on.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_node(&self, prompt: &mut Prompt, node: &str) -> anyhow::Result<()> {
        let node = prompt.get_node_by_id_mut(node).unwrap();
        self.set_value(node)
    }

    /// Sets the value on the given `Node`.
    ///
    /// # Inputs
    ///
    /// * `node` - A mutable reference to a `Node`.
    ///
    /// # Returns
    ///
    /// `Ok(())`` on success, or an error if the node could not be found.
    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()>;

    /// Finds a `Node` leading into the given `output_node`.
    ///
    /// # Inputs
    ///
    /// * `prompt` - A mutable reference to a `Prompt`.
    ///
    /// # Returns
    ///
    /// The id of the node on success, or `None` if the node could not be found.
    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        find_node::<N>(prompt, output_node)
    }
}

/// A `Setter` for setting the prompt text.
pub struct PromptSetter {
    /// The prompt text.
    pub prompt: String,
}

impl Setter<String, CLIPTextEncode> for PromptSetter {
    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()> {
        *as_node_mut::<CLIPTextEncode>(node)
            .context("Failed to cast node")?
            .text
            .value_mut()
            .context("Failed to get text value")? = self.prompt.clone();
        Ok(())
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        if let Some(node) = find_node::<KSampler>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&KSampler> {
                return Some(node.positive.node_id.clone());
            }
        }
        if let Some(node) = find_node::<SamplerCustom>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&SamplerCustom> {
                return Some(node.positive.node_id.clone());
            }
        }
        None
    }
}

impl From<String> for PromptSetter {
    fn from(prompt: String) -> Self {
        Self { prompt }
    }
}

/// A `Setter` for setting the negative prompt text.
pub struct NegativePromptSetter {
    /// The negative prompt text.
    pub prompt: String,
}

impl Setter<String, CLIPTextEncode> for NegativePromptSetter {
    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()> {
        PromptSetter::from(self).set_value(node)
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        if let Some(node) = find_node::<KSampler>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&KSampler> {
                return Some(node.negative.node_id.clone());
            }
        }
        if let Some(node) = find_node::<SamplerCustom>(prompt, output_node) {
            if let Ok(node) = prompt.get_node(&node) as anyhow::Result<&SamplerCustom> {
                return Some(node.negative.node_id.clone());
            }
        }
        None
    }
}

impl From<String> for NegativePromptSetter {
    fn from(prompt: String) -> Self {
        Self { prompt }
    }
}

impl From<&NegativePromptSetter> for PromptSetter {
    fn from(negative_prompt: &NegativePromptSetter) -> Self {
        Self {
            prompt: negative_prompt.prompt.clone(),
        }
    }
}

/// A `Setter` for setting the model.
pub struct ModelSetter {
    /// The model.
    pub(crate) model: String,
}

impl Setter<String, CheckpointLoaderSimple> for ModelSetter {
    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()> {
        *as_node_mut::<CheckpointLoaderSimple>(node)
            .context("Failed to cast node")?
            .ckpt_name
            .value_mut()
            .context("Failed to get ckpt_name value")? = self.model.clone();
        Ok(())
    }
}

impl From<String> for ModelSetter {
    fn from(model: String) -> Self {
        Self { model }
    }
}

/// A `Setter` for setting the image size.
pub struct SizeSetter {
    /// The width of the image.
    pub width: u32,
    /// The height of the image.
    pub height: u32,
}

impl Setter<(u32, u32), EmptyLatentImage> for SizeSetter {
    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()> {
        if self.width > 0 {
            *as_node_mut::<EmptyLatentImage>(node)
                .context("Failed to cast node")?
                .width
                .value_mut()
                .context("Failed to get width value")? = self.width;
        }
        if self.height > 0 {
            *as_node_mut::<EmptyLatentImage>(node)
                .context("Failed to cast node")?
                .height
                .value_mut()
                .context("Failed to get height value")? = self.height;
        }
        Ok(())
    }
}

impl From<(u32, u32)> for SizeSetter {
    fn from((width, height): (u32, u32)) -> Self {
        Self { width, height }
    }
}

/// A `Setter` for setting the seed. Generic over the node type.
pub struct SeedSetterT<N>
where
    N: Node + 'static,
{
    /// The seed.
    pub seed: i64,
    pub _phantom: std::marker::PhantomData<N>,
}

impl Setter<i64, KSampler> for SeedSetterT<KSampler> {
    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()> {
        *as_node_mut::<KSampler>(node)
            .context("Failed to cast node")?
            .seed
            .value_mut()
            .context("Failed to get seed value")? = self.seed;
        Ok(())
    }
}

impl Setter<i64, SamplerCustom> for SeedSetterT<SamplerCustom> {
    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()> {
        *as_node_mut::<SamplerCustom>(node)
            .context("Failed to cast node")?
            .noise_seed
            .value_mut()
            .context("Failed to get seed value")? = self.seed;
        Ok(())
    }
}

impl<N> From<i64> for SeedSetterT<N>
where
    N: Node + 'static,
{
    fn from(seed: i64) -> Self {
        Self {
            seed,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// A `Setter` for setting the seed.
pub type SeedSetter = DelegatingSetter<
    SeedSetterT<KSampler>,
    SeedSetterT<SamplerCustom>,
    i64,
    KSampler,
    SamplerCustom,
>;

/// A `Setter` that delegates to two other `Setter`s.
pub struct DelegatingSetter<S1, S2, T, N1, N2>
where
    S1: Setter<T, N1>,
    S2: Setter<T, N2>,
    N1: Node + 'static,
    N2: Node + 'static,
    T: Clone,
{
    /// The value to set.
    value: T,
    _phantom: std::marker::PhantomData<(S1, S2, N1, N2)>,
}

impl<S1, S2, T, N1, N2> Setter<T, N1> for DelegatingSetter<S1, S2, T, N1, N2>
where
    S1: Setter<T, N1>,
    S2: Setter<T, N2>,
    N1: Node + 'static,
    N2: Node + 'static,
    T: Clone,
{
    fn set(&self, prompt: &mut Prompt) -> anyhow::Result<()> {
        S1::from(self.value.clone()).set(prompt).or_else(|_| {
            S2::from(self.value.clone())
                .set(prompt)
                .context("Failed to set value")
        })
    }

    fn set_from(&self, prompt: &mut Prompt, output_node: &str) -> anyhow::Result<()> {
        S1::from(self.value.clone())
            .set_from(prompt, output_node)
            .or_else(|_| {
                S2::from(self.value.clone())
                    .set_from(prompt, output_node)
                    .context("Failed to set value")
            })
    }

    fn set_node(&self, prompt: &mut Prompt, node: &str) -> anyhow::Result<()> {
        S1::from(self.value.clone())
            .set_node(prompt, node)
            .or_else(|_| {
                S2::from(self.value.clone())
                    .set_node(prompt, node)
                    .context("Failed to set value")
            })
    }

    fn set_value(&self, node: &mut dyn Node) -> anyhow::Result<()> {
        S1::from(self.value.clone()).set_value(node).or_else(|_| {
            S2::from(self.value.clone())
                .set_value(node)
                .context("Failed to set value")
        })
    }

    fn find_node(prompt: &Prompt, output_node: Option<&str>) -> Option<String> {
        find_node::<N1>(prompt, output_node).or_else(|| find_node::<N2>(prompt, output_node))
    }
}

impl<S1, S2, T, N1, N2> From<T> for DelegatingSetter<S1, S2, T, N1, N2>
where
    S1: Setter<T, N1>,
    S2: Setter<T, N2>,
    N1: Node + 'static,
    N2: Node + 'static,
    T: Clone,
{
    fn from(value: T) -> Self {
        Self {
            value,
            _phantom: std::marker::PhantomData,
        }
    }
}
