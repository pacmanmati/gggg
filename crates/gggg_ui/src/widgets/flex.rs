use std::{cell::RefCell, collections::HashMap, f32::INFINITY, rc::Rc};

use crate::{context::Context, shape::UIShape, Offset};

use super::{widget::Widget, BoxConstraints, Size};

#[derive(Clone, Copy)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub enum AxisAlignment {
    Center,
    SpaceBetween,
    SpaceAround,
    Start,
    End,
}

#[derive(Clone, Copy)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Center,
    Stretch,
}

#[derive(Clone, Copy)]
pub enum AxisExtent {
    Min,
    Max,
}

#[derive(Clone)]
enum Child {
    Flex { widget: Box<dyn Widget>, flex: u32 },
    Fixed { widget: Box<dyn Widget> },
}

impl Child {
    pub fn widget(&self) -> &Box<dyn Widget> {
        match self {
            Child::Flex { widget, flex } => widget,
            Child::Fixed { widget } => widget,
        }
    }

    pub fn widget_mut(&mut self) -> &mut Box<dyn Widget> {
        match self {
            Child::Flex { widget, flex } => widget,
            Child::Fixed { widget } => widget,
        }
    }
}

#[derive(Clone)]
pub struct ChildCache {
    size_map: HashMap<usize, f32>,
    offset_map: HashMap<usize, f32>,
}

#[derive(Clone)]
pub struct Flex {
    axis: Axis,
    main_axis_alignment: AxisAlignment,
    main_axis_extent: AxisExtent,
    children: Vec<Child>,
    child_cache: Option<ChildCache>,
}

impl Flex {
    pub fn new() -> Self {
        Self {
            axis: Axis::Horizontal,
            main_axis_alignment: AxisAlignment::Start,
            children: Vec::new(),
            main_axis_extent: AxisExtent::Max,
            child_cache: None,
        }
    }

    pub fn with_axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    pub fn with_main_axis_alignment(mut self, alignment: AxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    pub fn with_fixed_child(mut self, widget: Box<dyn Widget>) -> Self {
        self.children.push(Child::Fixed { widget });
        self
    }

    pub fn with_flex_child(mut self, widget: Box<dyn Widget>, flex: u32) -> Self {
        self.children.push(Child::Flex { widget, flex });
        self
    }

    pub fn with_main_axis_extent(mut self, extent: AxisExtent) -> Self {
        self.main_axis_extent = extent;
        self
    }

    // axis agnostic
    fn center_strategy(
        &self,
        size_map: &HashMap<usize, f32>,
        constraints: BoxConstraints,
    ) -> HashMap<usize, f32> {
        match self.main_axis_extent {
            AxisExtent::Min => size_map.clone(), // yippie no work to do
            AxisExtent::Max => {
                let mut offset_map = size_map.clone();
                let total_child_span: f32 = size_map.values().sum();
                let child_center = total_child_span / 2.0;
                let flex_span_center = constraints.max_on_axis(self.axis) / 2.0;
                let offset_to_center = flex_span_center - child_center;
                offset_map.iter_mut().for_each(|(_, val)| {
                    *val += offset_to_center;
                });
                offset_map
            }
        }
    }

    fn start_strategy(
        &self,
        size_map: &HashMap<usize, f32>,
        constraints: BoxConstraints,
    ) -> HashMap<usize, f32> {
        let mut offset_map = HashMap::from([(0, 0.0)]);
        let mut stride = 0.0;
        for idx in 1..size_map.len() {
            stride += *size_map.get(&(idx - 1)).unwrap();
            offset_map.insert(idx, stride);
        }
        println!("size_map: {:?}", size_map);
        println!("offset_map: {:?}", offset_map);
        offset_map
    }

    fn end_strategy(
        &self,
        size_map: &HashMap<usize, f32>,
        constraints: BoxConstraints,
    ) -> HashMap<usize, f32> {
        match self.main_axis_extent {
            AxisExtent::Min => size_map.clone(),
            AxisExtent::Max => {
                let mut offset_map = size_map.clone();
                let total_child_span: f32 = size_map.values().sum();
                let flex_span = constraints.max_on_axis(self.axis);
                let offset_to_end = flex_span - total_child_span;
                offset_map.iter_mut().for_each(|(_, val)| {
                    *val += offset_to_end;
                });
                offset_map
            }
        }
    }
}

impl Default for Flex {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Flex {
    fn clone_dyn(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }

    // the layout call figures out where children will go (maintaining an internal state) but doesn't 'actually place them there'
    fn layout(&mut self, constraints: BoxConstraints, context: Rc<RefCell<Context>>) -> Size {
        // laying out a flex will require it's children to be laid out first
        // first we want to lay out the fixed children, we'll then split any remaining space among flex children but throw an error if there isn't enough space for their min size

        let fixed_children = self
            .children
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, child)| match child {
                Child::Fixed { widget } => Some((idx, widget.layout(constraints, context.clone()))),
                Child::Flex { .. } => None,
            })
            .collect::<Vec<_>>();

        let flex_children = self
            .children
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, child)| match child {
                Child::Fixed { .. } => None,
                Child::Flex { widget, flex } => Some((
                    idx,
                    widget.layout(constraints, context.clone()),
                    *flex,
                    widget.get_constraints(),
                )),
            })
            .collect::<Vec<_>>();

        let available_flex_space = constraints.max_on_axis(self.axis)
            - fixed_children.iter().fold(0.0, |acc, (_, child)| {
                acc + match self.axis {
                    Axis::Horizontal => child.width,
                    Axis::Vertical => child.height,
                }
            });
        let child_span_map = flex_to_size_map(&flex_children, available_flex_space, self.axis);
        let size_map = insert_fixed_children_to_map(&fixed_children, child_span_map, self.axis);

        let offset_map = match self.main_axis_alignment {
            // the logic for centering will look a little something like:
            // - find out where the leftmost of our children will be
            // - e.g. sum of child widths / 2 subtracted from the row's center
            // - place children one after another
            // - this step will be the same for both axes if we abstract direction away
            AxisAlignment::Center => self.center_strategy(&size_map, constraints),
            AxisAlignment::SpaceBetween => self.center_strategy(&size_map, constraints),
            AxisAlignment::SpaceAround => self.center_strategy(&size_map, constraints),
            AxisAlignment::Start => self.start_strategy(&size_map, constraints),
            AxisAlignment::End => self.end_strategy(&size_map, constraints),
        };

        let main_axis_size = size_map.values().sum();

        self.child_cache = Some(ChildCache {
            size_map,
            offset_map,
        });

        // TODO: layout in cross axis, split the two into separate functions?
        // for now, let's just assume that cross axis alignment is start
        // which also means that cross axis extent will equal cross axis extent of the largest child

        let max_cross_flex = flex_children
            .iter()
            .map(|child| match self.axis {
                Axis::Horizontal => child.1.width,
                Axis::Vertical => child.1.height,
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let max_cross_fixed = fixed_children
            .iter()
            .map(|child| match self.axis {
                Axis::Horizontal => child.1.width,
                Axis::Vertical => child.1.height,
            })
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let cross_axis_size = max_cross_flex.max(max_cross_fixed);

        match self.axis {
            Axis::Horizontal => Size {
                width: main_axis_size,
                height: cross_axis_size,
            },
            Axis::Vertical => Size {
                width: cross_axis_size,
                height: main_axis_size,
            },
        }
    }

    fn get_shapes(
        &mut self,
        offset: &Offset,
        constraints: BoxConstraints,
        context: Rc<RefCell<Context>>,
    ) -> Vec<UIShape> {
        if let Some(child_cache) = &self.child_cache {
            let mut shapes = Vec::new();
            for (idx, child) in self.children.iter_mut().enumerate() {
                let off = *child_cache.offset_map.get(&idx).unwrap();
                let position = match self.axis {
                    Axis::Horizontal => Offset { dx: off, dy: 0.0 },
                    Axis::Vertical => Offset { dx: 0.0, dy: off },
                };

                let sz = *child_cache.size_map.get(&idx).unwrap();
                let child_size = child.widget_mut().layout(constraints, context.clone());
                let new_size = match self.axis {
                    Axis::Horizontal => Size {
                        width: sz,
                        height: child_size.height,
                    },
                    Axis::Vertical => Size {
                        width: child_size.width,
                        height: sz,
                    },
                };

                let child_constraints = BoxConstraints::tight(new_size);

                for shape in child.widget_mut().get_shapes(
                    &(&position + offset),
                    child_constraints,
                    context.clone(),
                ) {
                    shapes.push(shape);
                }
            }

            shapes
        } else {
            panic!()
        }
    }

    fn get_constraints(&self) -> Option<BoxConstraints> {
        None
    }
}

// problem: suppose we've allocated space for all of our flex children, but at least one has a max size.
// we divide up the available space between our children, but discover that one of our max size children won't use all of the given space.
// this means that more space is available to divide up among the rest than we initially thought, we now have to repeat this step (up to N times in the worst case).
// addendum: we also need to respect flex children's min sizes.
// solution:
// - we need to identify which flex items have a min size and reserve their minimum size ahead of any calculations
// - now that we've got an idea of how much space we can actually share between our flex children, we need to identify max size children and see if they will be given more space than they actually need. if so, set them to max size, and add any excess back onto the available shared space.
// - finally work out how much space the remaining (unconstrained) items will get
fn flex_to_size_map(
    children: &Vec<(usize, Size, u32, Option<BoxConstraints>)>,
    available_space: f32,
    axis: Axis,
) -> HashMap<usize, f32> {
    let mut map = HashMap::new();
    let mut remaining_space = available_space;
    let min_children = children.iter().filter(|(_, _, _, constraints)| {
        constraints.is_some_and(|constraints| match axis {
            Axis::Horizontal => constraints.min.width > 0.0,
            Axis::Vertical => constraints.min.height > 0.0,
        })
    });
    min_children.for_each(|(_, _, _, constraints)| {
        remaining_space -= match axis {
            Axis::Horizontal => constraints.unwrap().min.width,
            Axis::Vertical => constraints.unwrap().min.height,
        };
    });

    if remaining_space <= 0.0 {
        panic!(); // overflow
    }

    let max_children = children
        .iter()
        .filter_map(|(idx, size, flex, constraints)| {
            constraints.and_then(|constraints| match axis {
                Axis::Horizontal => {
                    if constraints.max.width < INFINITY {
                        Some((
                            *idx,
                            *size,
                            *flex,
                            constraints.max.width,
                            constraints.min.width,
                        ))
                    } else {
                        None
                    }
                }
                Axis::Vertical => {
                    if constraints.max.height < INFINITY {
                        Some((
                            *idx,
                            *size,
                            *flex,
                            constraints.max.height,
                            constraints.min.width,
                        ))
                    } else {
                        None
                    }
                }
            })
        });

    // figure out whether a child will receive more space than it needs, if so we can add it back on to the remaining space.
    // min children have already been given their minimum space
    let mut max_children = max_children.collect::<Vec<(usize, Size, u32, f32, f32)>>();

    max_children.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());

    let flex_sum = children.iter().fold(0, |acc, (_, _, flex, _)| acc + flex);

    max_children.iter().for_each(|(_, _, flex, max, min)| {
        let space = *flex as f32 / flex_sum as f32 * remaining_space + min;
        if space > *max {
            remaining_space += space - max
        }
    });

    for (idx, _, flex, constraints) in children {
        let frac = *flex as f32 / flex_sum as f32;
        let sz = remaining_space * frac
            + constraints.map_or(0.0, |constraints| constraints.min_on_axis(axis));
        // check if there's enough space
        match axis {
            Axis::Horizontal => {
                // if size.width <= sz {
                map.insert(*idx, sz);
                // } else {
                //     panic!();
                // }
            }
            Axis::Vertical => {
                // if size.height <= sz {
                map.insert(*idx, sz);
                // } else {
                //     panic!();
                // }
            }
        }
    }
    map
}

fn insert_fixed_children_to_map(
    children: &Vec<(usize, Size)>,
    mut map: HashMap<usize, f32>,
    axis: Axis,
) -> HashMap<usize, f32> {
    for (idx, size) in children {
        map.insert(
            *idx,
            match axis {
                Axis::Horizontal => size.width,
                Axis::Vertical => size.height,
            },
        );
    }
    map
}
