use cgmath::Vector3;

//do I just make a Color Type?
//even though it's a little bit tedious im making it a trait because later there will likely
//be instances where I want to be able to run these functions on all light sources
//so that I can have a lighting manager passing these between each other and objects
pub trait LightTrait {
    fn is_on(&self) -> bool{
        true
    }
    fn get_emission_color(&self) -> &Vector3<f32>;
    fn get_emission_intensity(&self) -> &f32;//maybe just u8
    fn get_ambient_color(&self) -> &Vector3<f32>;
    fn get_specular_color(&self) -> &Vector3<f32>;
    fn get_position(&self) -> &Vector3<f32>;

    fn set_emission_color(&mut self, color: Vector3<f32>);
    fn set_emission_intensity(&mut self, intensity: f32);//maybe just u8
    fn set_ambient_color(&mut self, color: Vector3<f32>);
    fn set_specular_color(&mut self, color: Vector3<f32>);
    //maybe diffuse material color
    //i need a position but i am like 90% sure like 90%
    //that mesh or like worldcoords already has that.
    //so that will probably be the implimentation of it under most situations
}

//need light manager eventaully, which will prolly need mutex or sometype of shared
//like managment 😥😥😥... maybe not actaully
//it will be faster for the graphics card to process the combining of the lights
//all that it would need to do then is generate a matrix of lights and their elements
//this would be lit prolly


//ok after a lot of cumtimplation we finna use forward+ shading

pub struct LightManager{
    light_sources: Vec<Box<dyn LightTrait>>,//im wondering if I should have been using box more
}//erm intuitevly, I could just make this a hashmap and then just replace the value with a new one in an instance of modification...
//this might be unifficient

impl LightManager{
    pub fn new() -> Self{
        Self { light_sources: Vec::new() }
    }

    pub fn add_light(&mut self, light: Box<dyn LightTrait>){
        self.light_sources.push(light);
    }

    //pub fn compile_lights(&self,)
}
