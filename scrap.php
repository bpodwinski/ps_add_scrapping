<?php
ini_set('display_errors', 1);
ini_set('display_startup_errors', 1);
error_reporting(E_ALL);

require __DIR__ . '/config.php';
require __DIR__ . '/wp-load.php';

// En-têtes de la requête
$headers = array(
    "Content-Type: application/json"
);

// Données à envoyer
$data = array(
    "cmd" => "request.get",
    "url" => "***",
    "maxTimeout" => 60000
);

// Options de cURL
$options = array(
    CURLOPT_URL => $url_flare_proxy,
    CURLOPT_RETURNTRANSFER => true,
    CURLOPT_HTTPHEADER => $headers,
    CURLOPT_POST => true,
    CURLOPT_POSTFIELDS => json_encode($data),
    CURLOPT_TIMEOUT => 60 // Timeout en secondes
);

// Initialisation de cURL
$ch = curl_init();
curl_setopt_array($ch, $options);

// Exécution de la requête et récupération de la réponse
$response = curl_exec($ch);
$data = json_decode($response);
$responseSolution = $data->solution;
$responseHtml = $data->solution->response;
//file_put_contents("./response.txt", $responseHtml);
curl_close($ch);

$dom = new DOMDocument();
libxml_use_internal_errors(true); // Désactive les erreurs liées au chargement de HTML mal formé
$dom->loadHTML($responseHtml);
libxml_clear_errors(); // Nettoie les erreurs après le chargement

$xpath = new DOMXPath($dom);

// Get URL
if (!empty($responseSolution->url)) {
    $ps_url = $responseSolution->url;
    echo "PS URL: " . $ps_url . "<br>";
} else {
    echo "Pas de PS URL.";
}

// Get id_product
$id_product = $xpath->query("//script[contains(text(), 'psuser_assistance_track_event_params')]");
if ($id_product->length > 0) {
    $scriptContent = $id_product->item(0)->nodeValue;

    if (preg_match('/"id_product":(\d+)/', $scriptContent, $matches)) {
        $id_product = $matches[1];
        echo "ID: " . $id_product . "<br>";
    } else {
        echo "id_product n'a pas été trouvé.";
    }
}

// Get price ht
if (preg_match('/"price":(\d+(\.\d+)?)/', $responseHtml, $matches)) {
    $price_ht = $matches[1];
    echo "Prix: $price_ht" . "<br>";
} else {
    echo "Prix non trouvé.";
}

// Get module title
$title = $xpath->query('//title');
if ($title->length > 0) {
    $title = $title->item(0)->textContent; // Prend le contenu du premier nœud <title> trouvé
    echo "Title: $title" . "<br>";
} else {
    echo "Aucune balise <title> trouvée dans le HTML.";
}

// Get developper name
$dev_name = $xpath->query("//a[@id='ps_link_manufacturer']");
if ($dev_name->length > 0) {
    $dev_name = $dev_name->item(0)->getAttribute('title');
    echo "Developper: $dev_name" . "<br>";
} else {
    echo "Aucun élément avec l'ID 'ps_link_manufacturer' trouvé.";
}

// Get module version
$module_version = $xpath->query("//span[contains(@class, 'module__title-version')]");
if ($module_version->length > 0) {
    $module_version = $module_version->item(0)->textContent;
    echo "Module version: $module_version" . "<br>";
} else {
    echo "Aucune balise span avec la classe 'module__title-version' trouvée.";
}

// Get prestashop version requise
$divNodes = $xpath->query("//div[contains(text(), 'Version de PrestaShop requise')]");
if ($divNodes->length > 0) {
    $targetDiv = $divNodes->item(0)->nextSibling; // Trouver le nœud suivant

    // Assurer que nous sautons les nœuds de texte vides
    while ($targetDiv && $targetDiv->nodeType !== XML_ELEMENT_NODE) {
        $targetDiv = $targetDiv->nextSibling;
    }

    if ($targetDiv && $targetDiv->nodeType === XML_ELEMENT_NODE) {
        $ps_version_required = $targetDiv->textContent;
        echo "Version de PrestaShop requise: " . $ps_version_required . "<br>";
    } else {
        echo "Aucune div suivante trouvée ou valide.";
    }
} else {
    echo "Aucune div contenant 'Version de PrestaShop requise' trouvée.";
}

// Get meta description
$metaNodes = $xpath->query("//meta[@name='description']");
if ($metaNodes->length > 0) {
    // Récupérer la valeur de l'attribut 'content' du premier nœud <meta> trouvé
    $meta_description = $metaNodes->item(0)->getAttribute('content');
    echo "Balise meta: $meta_description" . "<br>";
} else {
    echo "Aucune balise meta avec l'attribut name='description' trouvée.";
}

//Get breadcrumb
function get_breadcrumbs($responseHtml) {
    $dom = new DOMDocument();
    libxml_use_internal_errors(true); // Pour ignorer les erreurs de HTML mal formé
    $dom->loadHTML($responseHtml); // Charger le contenu HTML
    libxml_clear_errors(); // Nettoyer les erreurs accumulées
    $xpath = new DOMXPath($dom);

    $scriptNodes = $xpath->query("//script[@type='application/ld+json']");
    $breadcrumbs = [];

    foreach ($scriptNodes as $node) {
        $json = trim($node->nodeValue);
        $data = json_decode($json, true);

        if (isset($data['@type']) && $data['@type'] === 'BreadcrumbList') {
            foreach ($data['itemListElement'] as $element) {
                $breadcrumbs[] = [
                    'position' => $element['position'],
                    'id' => $element['item']['@id'],
                    'name' => $element['item']['name']
                ];
            }
            break; // Quitter la boucle une fois les breadcrumbs trouvés et traités
        }
    }

    return $breadcrumbs;
}

// Get description
$divNodes = $xpath->query("//div[contains(@class, 'ui-text-collapse collapsed product-description__content puik-body-large')]");
if ($divNodes->length > 0) {
    $innerHTML = '';
    $div = $divNodes->item(0); // Prendre la première div trouvée
    $children = $div->childNodes;

    foreach ($children as $child) {
        $innerHTML .= $dom->saveHTML($child);
    }

    $description = str_replace(["<div>", "</div>"], "", $innerHTML);

    echo "Description: " . $description . "\n";
} else {
    echo "Aucune div avec la classe 'product-description__content' trouvée";
    $description = "";
}

// Get caracteristiques
$divNodes = $xpath->query("//div[contains(@class, 'ui-text-collapse collapsed product-description__content puik-body-large')]");
if ($divNodes->length > 0) {
    $innerHTML = '';
    $div = $divNodes->item(1); // Prendre la première div trouvée
    $children = $div->childNodes;

    foreach ($children as $child) {
        $innerHTML .= $dom->saveHTML($child);
    }

    $caracteristiques = str_replace(["<div>", "</div>"], "", $innerHTML);

    echo "Caractéristiques: " . $caracteristiques . "<br>";
} else {
    echo "Aucune div avec la classe 'product-description__content' trouvée.";
}

// Get Compatibilité multiboutique
$divNodes = $xpath->query("//div[contains(text(), 'Compatibilité multiboutique')]");
if ($divNodes->length > 0) {
    $targetDiv = $divNodes->item(0)->nextSibling; // Trouver le nœud suivant

    // Assurer que nous sautons les nœuds de texte vides
    while ($targetDiv && $targetDiv->nodeType !== XML_ELEMENT_NODE) {
        $targetDiv = $targetDiv->nextSibling;
    }

    if ($targetDiv && $targetDiv->nodeType === XML_ELEMENT_NODE) {
        $is_multistore = $targetDiv->textContent;
        echo "Compatibilité multiboutique: " . $is_multistore . "<br>";
    } else {
        echo "Aucune div suivante trouvée ou valide.";
    }
} else {
    echo "Aucune div contenant 'Compatibilité multiboutique' trouvée.";
}

// Get Contient des surcharges
$divNodes = $xpath->query("//div[contains(text(), 'Contient des surcharges')]");

if ($divNodes->length > 0) {
    $targetDiv = $divNodes->item(0)->nextSibling; // Trouver le nœud suivant

    // Assurer que nous sautons les nœuds de texte vides
    while ($targetDiv && $targetDiv->nodeType !== XML_ELEMENT_NODE) {
        $targetDiv = $targetDiv->nextSibling;
    }

    if ($targetDiv && $targetDiv->nodeType === XML_ELEMENT_NODE) {
        $as_overrides = $targetDiv->textContent;
        echo "Contient des surcharges: " . $as_overrides . "<br>";
    } else {
        echo "Aucune div suivante trouvée ou valide.";
    }
} else {
    echo "Aucune div contenant 'Contient des surcharges' trouvée.";
}

// Get Dernière mise à jour
$divNodes = $xpath->query("//div[contains(text(), 'Dernière mise à jour')]");

if ($divNodes->length > 0) {
    $targetDiv = $divNodes->item(0)->nextSibling; // Trouver le nœud suivant

    // Assurer que nous sautons les nœuds de texte vides
    while ($targetDiv && $targetDiv->nodeType !== XML_ELEMENT_NODE) {
        $targetDiv = $targetDiv->nextSibling;
    }

    if ($targetDiv && $targetDiv->nodeType === XML_ELEMENT_NODE) {
        $last_update = $targetDiv->textContent;
        echo "Dernière mise à jour: " . $last_update . "<br>";
    } else {
        echo "Aucune div suivante trouvée ou valide.";
    }
} else {
    echo "Aucune div contenant 'Dernière mise à jour' trouvée.";
}

// Get Date de publication
$divNodes = $xpath->query("//div[contains(text(), 'Date de publication')]");

if ($divNodes->length > 0) {
    $targetDiv = $divNodes->item(0)->nextSibling; // Trouver le nœud suivant

    // Assurer que nous sautons les nœuds de texte vides
    while ($targetDiv && $targetDiv->nodeType !== XML_ELEMENT_NODE) {
        $targetDiv = $targetDiv->nextSibling;
    }

    if ($targetDiv && $targetDiv->nodeType === XML_ELEMENT_NODE) {
        $date = $targetDiv->textContent;
        echo "Date de publication: " . $date . "<br>";
    } else {
        echo "Aucune div suivante trouvée ou valide.";
    }
} else {
    echo "Aucune div contenant 'Date de publication' trouvée.";
}

// Get images
$imgNodes = $xpath->query("//img");
$images_url = [];
foreach ($imgNodes as $img) {
    $src = $img->getAttribute('src');
    if (strpos($src, 'https://addons.prestashop.com/') !== false) {
        $images_url[] = $src;
        echo "Lien trouvé dans src: $src<br>";
    }
}

// Fonction pour télécharger l'image et obtenir l'ID de la pièce jointe
function upload_image_to_media_library($image_url) {
    require_once(ABSPATH . 'wp-admin/includes/image.php');
    require_once(ABSPATH . 'wp-admin/includes/file.php');
    require_once(ABSPATH . 'wp-admin/includes/media.php');

    // Téléchargez l'image à partir de l'URL
    $tmp = download_url($image_url);

    // Vérifiez s'il y a des erreurs lors du téléchargement
    if (is_wp_error($tmp)) {
        return 0;
    }

    // Préparez un tableau des données du fichier
    $file_array = [
        'name'     => basename($image_url),
        'tmp_name' => $tmp
    ];

    // Vérifiez les erreurs du fichier avant de continuer
    $attachment_id = media_handle_sideload($file_array, 0);

    // Si une erreur se produit lors de l'upload, effacez le fichier temporaire
    if (is_wp_error($attachment_id)) {
        @unlink($file_array['tmp_name']);
        return 0;
    }

    return $attachment_id;
}

// Fonction pour trouver une page par son nom
function find_page_id_by_title($title) {
    $page = get_page_by_title($title, OBJECT, 'page');
    return $page ? $page->ID : 0;
}

// Fonction pour créer ou mettre à jour les pages récursivement
function create_page($page_data, $parent_id) {
    global $id_product ;
    global $price_ht;
    global $title;
    global $dev_name;
    global $module_version;
    global $date;
    global $last_update;
    global $ps_version_required;
    global $as_overrides;
    global $is_multistore;
    global $description;
    global $caracteristiques;
    global $ps_url;
    global $images_url;

    $page_content = '';

    if (!empty($images_url)) {
        foreach ($images_url as $image_url) {
            $attachment_id = upload_image_to_media_library($image_url);
            if ($attachment_id) {
                $image_tag = wp_get_attachment_image($attachment_id, 'full');
                $img_tags .= '[fusion_image linktarget="_self" image="' . $image_url . '" image_id="' . $attachment_id . '|full" /]';
            } else {
                $img_tags .= "";
            }
        }
    }

    $page_id = find_page_id_by_title($page_data['name']);

    // Modèle de contenu
    $template = file_get_contents(__DIR__ . '/template.txt');

    $search = [
        '[ID_PS_PRODUCT]',
        '[PRICE_HT]',
        '[TITLE]',
        '[DEV_NAME]',
        '[MODULE_VERSION]',
        '[PUBLICATION_DATE]',
        '[LAST_UPDATE]',
        '[PRESTASHOP_VERSION]',
        '[AS_OVERRIDES]',
        '[IS_MULTISTORE]',
        '[DESCRIPTION]',
        '[CARACTERISTIQUES]',
        '#URL_MODULE',
        '[IMG_TAGS]',
    ];

    $replace = [
        $id_product,
        $price_ht,
        $title,
        $dev_name,
        $module_version,
        $date,
        $last_update,
        $ps_version_required,
        $as_overrides,
        $is_multistore,
        $description,
        $caracteristiques,
        $ps_url,
        $img_tags,
    ];

    $template = str_replace($search, $replace, $template);

    // Définissez les paramètres de la page
    $page = [
        'post_type'    => 'page',
        'post_title'   => $page_data['name'],
        'post_content' => $template,
        'post_status'  => 'publish',
        'post_author'  => 1,
        'post_parent'  => $parent_id,
    ];

    echo "parent_id: " . $parent_id . " - ";

    if ($page_id === 0) {
        // Insérez la page dans la base de données
        $page_id = wp_insert_post($page);

        if ($page_id) {
            echo 'Page créée avec succès avec l\'ID : ' . $page_id . ' et titre : ' . $page_data['name'] . '<br>';
        } else {
            echo 'Échec de la création de la page avec le titre : ' . $page_data['name'] . '<br>';
        }
    } else {
        // Mettre à jour la page existante
        $page['ID'] = $page_id;
        $updated_id = wp_update_post($page);

        if ($updated_id) {
            echo 'Page mise à jour avec succès avec l\'ID : ' . $page_id . ' et titre : ' . $page_data['name'] . '<br>';
        } else {
            echo 'Échec de la mise à jour de la page avec le titre : ' . $page_data['name'] . '<br>';
        }
    }

    return $page_id;
}

// Extraction des breadcrumbs et création des pages
$breadcrumbs_pages = get_breadcrumbs($responseHtml);

if (!empty($breadcrumbs_pages)) {
    $parent_id = $racine_id_page; // Initialiser avec 0 pour les pages de premier niveau

    foreach ($breadcrumbs_pages as $breadcrumb) {
        $page_data = [
            'name' => $breadcrumb['name'],
            'children' => []
        ];

        // Créer ou mettre à jour la page et obtenir l'ID de la page créée
        $parent_id = create_page($page_data, $parent_id);
    }
} else {
    echo 'Aucun breadcrumb trouvé.';
}
