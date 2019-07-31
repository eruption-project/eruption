jQuery(document).ready(function($) {
		// flexslider main
		$('#main-flexslider').flexslider({						
			animation: "swing",
			direction: "vertical", 
			slideshow: true,
			slideshowSpeed: 3500,
			animationDuration: 1000,
			directionNav: true,
			prevText: '<i class="icon-angle-up icon-2x"></i>',       
			nextText: '<i class="icon-angle-down icon-2x active"></i>', 
			controlNav: false,
			smootheHeight:true,						
			useCSS: false
		});

	$('.scrollup').click(function() {
		$("html, body").animate({ scrollTop: 0 }, 1000);
		return false;
	});
	
		$('.accordion').on('show', function (e) {
		
			$(e.target).prev('.accordion-heading').find('.accordion-toggle').addClass('active');
			$(e.target).prev('.accordion-heading').find('.accordion-toggle i').removeClass('icon-plus');
			$(e.target).prev('.accordion-heading').find('.accordion-toggle i').addClass('icon-minus');
		});
		
		$('.accordion').on('hide', function (e) {
			$(this).find('.accordion-toggle').not($(e.target)).removeClass('active');
			$(this).find('.accordion-toggle i').not($(e.target)).removeClass('icon-minus');
			$(this).find('.accordion-toggle i').not($(e.target)).addClass('icon-plus');
		});	

    // Localscrolling 
	$('#menu-main, .brand').localScroll();
	
	// cache container
	var $container = $('#portfolio-wrap');
	$.browser.safari = ($.browser.webkit && !(/chrome/.test(navigator.userAgent.toLowerCase())));	
	
	if($.browser.safari){ 	
	// initialize isotope
	$container.isotope({
		animationEngine : 'jquery',
		animationOptions: {
			duration: 200,
			queue: false
		},
		layoutMode: 'fitRows'
	});
	} else {	
	$container.isotope({
		animationEngine : 'best-available',
		animationOptions: {
			duration: 200,
			queue: false
		},
		layoutMode: 'fitRows'
	});	
	
	$(window).resize(function() {
		$container.isotope('reLayout');
	});
	}
	// filter items when filter link is clicked
	$('#filters a').click(function(){
		$('#filters a').removeClass('active');
		$(this).addClass('active');
		var selector = $(this).attr('data-filter');
		$container.isotope({ filter: selector });
		return false;
	});
});
