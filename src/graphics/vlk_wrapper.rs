use ash::{vk, Device};
use ash::vk::Handle;
use std::sync::Arc;
use std::ffi::c_void;

use super::gl_wrapper::{UniformValue, VertexAttribute};
use super::api_trait::{ShaderManagerApi, GraphicsBackend, GpuBuffer, VertexArray};

// ---------------- Vulkan Buffer Wrapper ----------------

pub struct VkBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    device: Arc<ash::Device>,
}

impl VkBuffer {
    pub fn new(
        device: Arc<ash::Device>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
    ) -> Self {
        unsafe {
            // Create buffer
            let buffer_info = vk::BufferCreateInfo {
                s_type: vk::StructureType::BUFFER_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::BufferCreateFlags::empty(),
                size,
                usage,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                queue_family_index_count: 0,
                p_queue_family_indices: std::ptr::null(),
                _marker: std::marker::PhantomData,
            };

            let buffer = device.create_buffer(&buffer_info, None).unwrap();
            let mem_requirements = device.get_buffer_memory_requirements(buffer);
            
            // Find memory type
            let mem_props = instance.get_physical_device_memory_properties(physical_device);
            let mut memory_type_index = 0;
            for i in 0..mem_props.memory_type_count {
                if (mem_requirements.memory_type_bits & (1 << i)) != 0
                    && (mem_props.memory_types[i as usize].property_flags & memory_properties) == memory_properties
                {
                    memory_type_index = i;
                    break;
                }
            }

            // Allocate memory
            let alloc_info = vk::MemoryAllocateInfo {
                s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
                p_next: std::ptr::null(),
                allocation_size: mem_requirements.size,
                memory_type_index,
                _marker: std::marker::PhantomData,
            };

            let memory = device.allocate_memory(&alloc_info, None).unwrap();
            device.bind_buffer_memory(buffer, memory, 0).unwrap();

            Self { buffer, memory, size, device }
        }
    }

    pub fn upload_data<T>(&self, data: &[T]) {
        unsafe {
            let data_ptr = self.device
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
                .unwrap();
            
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const c_void,
                data_ptr,
                std::mem::size_of_val(data),
            );

            self.device.unmap_memory(self.memory);
        }
    }
}

impl Drop for VkBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

impl GpuBuffer for VkBuffer {
    fn bind(&self) { /* no-op for vulkan */ }
    fn unbind(&self) { /* no-op */ }
    fn upload_f32(&self, data: &[f32]) { self.upload_data(data); }
    fn upload_i32(&self, data: &[i32]) { self.upload_data(data); }
    fn id(&self) -> u32 { self.buffer.as_raw() as u32 }
}

// ---------------- Vulkan "Vertex Array" (descriptor storage) ----------------

pub struct VkVertexArray {
    pub bindings: Vec<vk::VertexInputBindingDescription>,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
}

impl VertexArray for VkVertexArray {
    fn new() -> Self {
        Self {
            bindings: Vec::new(),
            attributes: Vec::new(),
        }
    }
    fn bind(&self) { /* no-op */ }
    fn unbind(&self) { /* no-op */ }
}

// ---------------- Vulkan Pipeline (shader) ----------------

pub struct VkPipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptor_layout: vk::DescriptorSetLayout,
    device: Arc<ash::Device>,
}

impl Drop for VkPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.layout, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_layout, None);
        }
    }
}

// ---------------- Vulkan Shader Manager ----------------

pub struct VkShaderManager {
    device: Arc<ash::Device>,
    physical_device: vk::PhysicalDevice,
    instance: Arc<ash::Instance>,
    render_pass: vk::RenderPass,
}

impl VkShaderManager {
    pub fn new(
        device: Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        instance: Arc<ash::Instance>,
        render_pass: vk::RenderPass,
    ) -> Self {
        Self { device, physical_device, instance, render_pass }
    }

    fn create_shader_module(&self, spirv_code: &[u32]) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: spirv_code.len() * 4,
            p_code: spirv_code.as_ptr(),
            _marker: std::marker::PhantomData,
        };

        unsafe {
            self.device.create_shader_module(&create_info, None).unwrap()
        }
    }
}

impl ShaderManagerApi for VkShaderManager {
    type ShaderHandle = Arc<VkPipeline>;

    fn load_shader(&mut self, _name: &str, vert_spirv: &str, frag_spirv: &str) -> Self::ShaderHandle {
        // NOTE: vert_spirv and frag_spirv should be base64 encoded SPIR-V or paths to .spv files
        // For now, this is a placeholder - you need actual SPIR-V bytes
        
        // Placeholder: convert strings to fake SPIR-V (you'd load actual .spv files)
        let vert_code: Vec<u32> = vec![]; // Load your .spv file here
        let frag_code: Vec<u32> = vec![]; // Load your .spv file here

        let vert_module = self.create_shader_module(&vert_code);
        let frag_module = self.create_shader_module(&frag_code);

        let entry_name = std::ffi::CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                stage: vk::ShaderStageFlags::VERTEX,
                module: vert_module,
                p_name: entry_name.as_ptr(),
                p_specialization_info: std::ptr::null(),
                _marker: std::marker::PhantomData,
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: std::ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: frag_module,
                p_name: entry_name.as_ptr(),
                p_specialization_info: std::ptr::null(),
                _marker: std::marker::PhantomData,
            },
        ];

        // Descriptor set layout for uniforms
        let ubo_binding = vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
            _marker: std::marker::PhantomData,
        };

        let layout_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: 1,
            p_bindings: &ubo_binding,
            _marker: std::marker::PhantomData,
        };

        let descriptor_layout = unsafe {
            self.device.create_descriptor_set_layout(&layout_info, None).unwrap()
        };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 1,
            p_set_layouts: &descriptor_layout,
            push_constant_range_count: 0,
            p_push_constant_ranges: std::ptr::null(),
            _marker: std::marker::PhantomData,
        };

        let pipeline_layout = unsafe {
            self.device.create_pipeline_layout(&pipeline_layout_info, None).unwrap()
        };

        // Vertex input (empty for now)
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_description_count: 0,
            p_vertex_binding_descriptions: std::ptr::null(),
            vertex_attribute_description_count: 0,
            p_vertex_attribute_descriptions: std::ptr::null(),
            _marker: std::marker::PhantomData,
        };

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            _marker: std::marker::PhantomData,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count: 1,
            p_viewports: std::ptr::null(),
            scissor_count: 1,
            p_scissors: std::ptr::null(),
            _marker: std::marker::PhantomData,
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.0,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 0.0,
            line_width: 1.0,
            _marker: std::marker::PhantomData,
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 1.0,
            p_sample_mask: std::ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE,
            _marker: std::marker::PhantomData,
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
        };

        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment,
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            _marker: std::marker::PhantomData,
        };

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineDynamicStateCreateFlags::empty(),
            dynamic_state_count: 2,
            p_dynamic_states: dynamic_states.as_ptr(),
            _marker: std::marker::PhantomData,
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: 2,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_info,
            p_input_assembly_state: &input_assembly,
            p_tessellation_state: std::ptr::null(),
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisampling,
            p_depth_stencil_state: std::ptr::null(),
            p_color_blend_state: &color_blending,
            p_dynamic_state: &dynamic_state,
            layout: pipeline_layout,
            render_pass: self.render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            _marker: std::marker::PhantomData,
        };

        let pipeline = unsafe {
            self.device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[pipeline_info],
                None,
            ).unwrap()[0]
        };

        unsafe {
            self.device.destroy_shader_module(vert_module, None);
            self.device.destroy_shader_module(frag_module, None);
        }

        Arc::new(VkPipeline {
            pipeline,
            layout: pipeline_layout,
            descriptor_layout,
            device: self.device.clone(),
        })
    }

    fn bind(&mut self, _shader: &Self::ShaderHandle) { /* happens in cmd buffer */ }
    fn unbind(&mut self) { /* no-op */ }
    
    fn set_uniform(&mut self, _shader: &Self::ShaderHandle, _name: &str, _value: &UniformValue) {
        // In Vulkan you'd update descriptor sets here
        // This is complex and depends on your uniform buffer setup
        eprintln!("VkShaderManager::set_uniform not fully implemented - use descriptor sets");
    }

    fn shader_id(&self, shader: &Self::ShaderHandle) -> Option<u32> {
        Some(shader.pipeline.as_raw() as u32)
    }
}

// ---------------- Vulkan Backend ----------------

pub struct VulkanBackend {
    pub shader_manager: VkShaderManager,
    pub device: Arc<ash::Device>,
    pub physical_device: vk::PhysicalDevice,
    pub instance: Arc<ash::Instance>,
    pub command_buffer: Option<vk::CommandBuffer>,
}

impl VulkanBackend {
    pub fn new(
        device: Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        instance: Arc<ash::Instance>,
        render_pass: vk::RenderPass,
    ) -> Self {
        let shader_manager = VkShaderManager::new(
            device.clone(),
            physical_device,
            instance.clone(),
            render_pass,
        );

        Self {
            shader_manager,
            device,
            physical_device,
            instance,
            command_buffer: None,
        }
    }

    pub fn set_command_buffer(&mut self, cmd_buffer: vk::CommandBuffer) {
        self.command_buffer = Some(cmd_buffer);
    }
}

impl GraphicsBackend for VulkanBackend {
    type VA = VkVertexArray;
    type Buffer = VkBuffer;
    type Shader = Arc<VkPipeline>;

    fn create_vertex_array(&mut self) -> Self::VA {
        VkVertexArray::new()
    }

    fn create_buffer(&mut self, target: u32, _usage: u32) -> Self::Buffer {
        let vk_usage = match target {
            gl::ARRAY_BUFFER => vk::BufferUsageFlags::VERTEX_BUFFER,
            gl::ELEMENT_ARRAY_BUFFER => vk::BufferUsageFlags::INDEX_BUFFER,
            gl::UNIFORM_BUFFER => vk::BufferUsageFlags::UNIFORM_BUFFER,
            _ => vk::BufferUsageFlags::VERTEX_BUFFER,
        };

        VkBuffer::new(
            self.device.clone(),
            1024 * 1024, // 1MB default
            vk_usage | vk::BufferUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            self.physical_device,
            &self.instance,
        )
    }

    fn create_vertex_attribute(&mut self,
        index: u32,
        size: i32,
        r#type: u32,
        normalized: u8,
        stride: i32,
        pointer: *const c_void
    ) -> VertexAttribute {
        let norm = if normalized == 0 { gl::FALSE } else { gl::TRUE };//todo totally useless btw
        VertexAttribute::new(index, size, r#type, norm, stride, pointer)
    }

    fn create_shader_from_src(&mut self, name: &str, vert: &str, frag: &str) -> Self::Shader {
        self.shader_manager.load_shader(name, vert, frag)
    }

    fn set_uniform(&mut self, shader: &Self::Shader, name: &str, value: &UniformValue) {
        self.shader_manager.set_uniform(shader, name, value)
    }

    fn draw_arrays(&mut self, _vao: &Self::VA, shader: &Self::Shader, count: i32) {
        if let Some(cmd) = self.command_buffer {
            unsafe {
                self.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, shader.pipeline);
                self.device.cmd_draw(cmd, count as u32, 1, 0, 0);
            }
        }
    }

    fn draw_elements(&mut self, _vao: &Self::VA, shader: &Self::Shader, count: i32) {
        if let Some(cmd) = self.command_buffer {
            unsafe {
                self.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, shader.pipeline);
                self.device.cmd_draw_indexed(cmd, count as u32, 1, 0, 0, 0);
            }
        }
    }
}