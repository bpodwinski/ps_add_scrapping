<?php
/*
Plugin Name: Custom Fields for Product Categories
Description: Adds a custom field to WooCommerce product categories.
Version: 1.0
Author: Test Test
*/

// Add fields during product category editing
add_action('product_cat_add_form_fields', 'add_custom_field_to_product_cat', 10, 1);
add_action('product_cat_edit_form_fields', 'edit_custom_field_in_product_cat', 10, 1);

function add_custom_field_to_product_cat($taxonomy) {
    ?>
    <div class="form-field">
        <label for="ps_addons_cat_id"><?php _e('PrestaShop Addons category id', 'woocommerce'); ?></label>
        <input type="number" name="ps_addons_cat_id" id="ps_addons_cat_id">
    </div>
    <?php
}

function edit_custom_field_in_product_cat($term) {
    $value = get_term_meta($term->term_id, 'ps_addons_cat_id', true);
    ?>
    <tr class="form-field">
        <th scope="row" valign="top"><label for="ps_addons_cat_id"><?php _e('PrestaShop Addons category id', 'woocommerce'); ?></label></th>
        <td>
            <input type="number" name="ps_addons_cat_id" id="ps_addons_cat_id" value="<?php echo esc_attr($value); ?>">
        </td>
    </tr>
    <?php
}

// Save the custom field data
add_action('created_product_cat', 'save_custom_field_data', 10, 2);
add_action('edited_product_cat', 'save_custom_field_data', 10, 2);

function save_custom_field_data($term_id) {
    if (isset($_POST['ps_addons_cat_id'])) {
        $int_value = intval($_POST['ps_addons_cat_id']);
        update_term_meta($term_id, 'ps_addons_cat_id', $int_value);
    }
}

// Ensure that the WooCommerce API includes the custom field in responses
add_filter('woocommerce_rest_prepare_product_cat', function ($response, $item, $request) {
    $ps_addons_cat_id = get_term_meta($item->term_id, 'ps_addons_cat_id', true);
    $response->data['ps_addons_cat_id'] = intval($ps_addons_cat_id);
    return $response;
}, 10, 3);

// Allows updating the custom field via the WooCommerce API
add_action('woocommerce_rest_insert_product_cat', function ($term, $request, $creating) {
    if (isset($request['ps_addons_cat_id'])) {
        if (is_numeric($request['ps_addons_cat_id'])) {
            $int_value = intval($request['ps_addons_cat_id']);
            update_term_meta($term->term_id, 'ps_addons_cat_id', $int_value);
        } else {
            return new WP_Error('invalid_type', 'The value of ps_addons_cat_id must be an integer', array('status' => 400));
        }
    }
}, 10, 3);

add_filter('woocommerce_rest_product_cat_query', function ($query, $request) {
    if (!empty($request['ps_addons_cat_id'])) {
        $int_value = intval($request['ps_addons_cat_id']);
        $query['meta_query'] = array(
            array(
                'key' => 'ps_addons_cat_id',
                'value' => $int_value,
                'compare' => '='
            )
        );
    }
    return $query;
}, 10, 2);

function authorize_custom_meta_rest_api() {
    // Register custom fields for WooCommerce products
    register_meta('product', 'ps_product_id', array(
        'show_in_rest' => true,
        'single' => true,
        'type' => 'string',
    ));

    register_meta('product', 'ps_product_url', array(
        'show_in_rest' => true,
        'single' => true,
        'type' => 'string',
    ));
}

function add_meta_query_filter() {
    // Add a query filter for WooCommerce products via REST API
    add_filter('rest_product_query', function ($args, $request) {
        $meta_key = $request->get_param('meta_key');
        $meta_value = $request->get_param('meta_value');
        if ($meta_key && $meta_value) {
            $args['meta_query'] = array(
                array(
                    'key'   => $meta_key,
                    'value' => $meta_value,
                )
            );
        }
        return $args;
    }, 10, 2);
}

add_action('rest_api_init', 'add_meta_query_filter');
add_action('init', 'authorize_custom_meta_rest_api');
