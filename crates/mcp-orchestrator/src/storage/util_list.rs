use kube::api::ListParams;

#[derive(Clone, Default)]
pub struct ListOption {
    pub first: Option<i32>,
    pub after: Option<String>,
}

impl ListOption {
    pub fn get_limit(&self) -> usize {
        self.first.unwrap_or(10) as usize
    }

    pub fn has_more(
        &self,
        metadata: &k8s_openapi::apimachinery::pkg::apis::meta::v1::ListMeta,
    ) -> bool {
        metadata.remaining_item_count.unwrap_or(0) > 0
    }

    pub fn to_list_param(&self, label_query: String) -> ListParams {
        ListParams {
            label_selector: Some(label_query),
            limit: Some(self.get_limit() as u32),
            continue_token: self.after.clone(),
            ..Default::default()
        }
    }
}
